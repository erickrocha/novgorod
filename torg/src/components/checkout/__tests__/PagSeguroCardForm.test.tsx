import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import PagSeguroCardForm from '../PagSeguroCardForm';

const CARD = '4111 1111 1111 1111';

function fill(container: HTMLElement) {
  const set = (id: string, value: string) => fireEvent.change(container.querySelector(`#${id}`)!, { target: { value } });
  set('pagseguro-card-number', CARD);
  set('pagseguro-card-holder', 'FULANO DE TAL');
  set('pagseguro-card-expiry', '1230');
  set('pagseguro-card-cvv', '123');
}

function setup(onPay = vi.fn().mockResolvedValue(undefined)) {
  const view = render(<PagSeguroCardForm publicKey="PK" amountCents={1000} email="a@b.c" cpf="52998224725" onPay={onPay} />);
  return { ...view, onPay };
}

describe('PagSeguroCardForm card data', () => {
  afterEach(() => { delete window.PagSeguro; });

  it('sends only the SDK-encrypted card', async () => {
    window.PagSeguro = { encryptCard: vi.fn().mockReturnValue({ encryptedCard: 'ENCRYPTED' }) };
    const { container, onPay } = setup();
    await waitFor(() => expect(container.querySelector('#pagseguro-submit')).not.toBeDisabled());
    fill(container);
    fireEvent.submit(container.querySelector('form')!);
    await waitFor(() => expect(onPay).toHaveBeenCalledTimes(1));
    const payload = JSON.stringify(onPay.mock.calls[0][0]);
    expect(onPay.mock.calls[0][0].token).toBe('ENCRYPTED');
    expect(payload).not.toContain('4111');
    expect(payload).not.toContain('123');
  });

  it('never sends the card when the SDK reports errors', async () => {
    window.PagSeguro = { encryptCard: vi.fn().mockReturnValue({ hasErrors: true, errors: [{}] }) };
    const { container, onPay } = setup();
    await waitFor(() => expect(container.querySelector('#pagseguro-submit')).not.toBeDisabled());
    fill(container);
    fireEvent.submit(container.querySelector('form')!);
    expect(await screen.findByText(/Dados do cartão inválidos/)).toBeInTheDocument();
    expect(onPay).not.toHaveBeenCalled();
  });

  it('keeps the form disabled and sends nothing when the SDK cannot load', async () => {
    const append = vi.spyOn(document.head, 'appendChild').mockImplementation(node => {
      queueMicrotask(() => (node as HTMLScriptElement).onerror?.(new Event('error')));
      return node;
    });
    const { container, onPay } = setup();
    expect(await screen.findByText(/ambiente seguro de pagamento/)).toBeInTheDocument();
    fill(container);
    fireEvent.submit(container.querySelector('form')!);
    expect(container.querySelector('#pagseguro-submit')).toBeDisabled();
    expect(onPay).not.toHaveBeenCalled();
    append.mockRestore();
  });
});

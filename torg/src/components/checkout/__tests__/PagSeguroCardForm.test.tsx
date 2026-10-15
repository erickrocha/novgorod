import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import PagSeguroCardForm from '../PagSeguroCardForm';
import checkoutService from '../../../services/checkoutService';

vi.mock('../../../services/checkoutService');

const PLAN = [
  { installments: 1, installmentCents: 1000, totalCents: 1000, interestFree: true },
  { installments: 2, installmentCents: 500, totalCents: 1000, interestFree: true },
  { installments: 3, installmentCents: 370, totalCents: 1110, interestFree: false },
];

const CARD = '4111 1111 1111 1111';

function fill(container: HTMLElement) {
  const set = (id: string, value: string) => fireEvent.change(container.querySelector(`#${id}`)!, { target: { value } });
  set('pagseguro-card-number', CARD);
  set('pagseguro-card-holder', 'FULANO DE TAL');
  set('pagseguro-card-expiry', '1230');
  set('pagseguro-card-cvv', '123');
}

function setup(onPay = vi.fn().mockResolvedValue(undefined)) {
  const view = render(<PagSeguroCardForm publicKey="PK" tenantId={7} amountCents={1000} email="a@b.c" cpf="52998224725" onPay={onPay} />);
  return { ...view, onPay };
}

describe('PagSeguroCardForm card data', () => {
  beforeEach(() => { vi.mocked(checkoutService.installments).mockResolvedValue(PLAN); });
  afterEach(() => { delete window.PagSeguro; vi.clearAllMocks(); });

  it('sends only the SDK-encrypted card', async () => {
    window.PagSeguro = { encryptCard: vi.fn().mockReturnValue({ encryptedCard: 'ENCRYPTED' }) };
    const { container, onPay } = setup();
    await waitFor(() => expect(container.querySelector('#pagseguro-submit')).not.toBeDisabled());
    fill(container);
    fireEvent.submit(container.querySelector('form')!);
    await waitFor(() => expect(onPay).toHaveBeenCalledTimes(1));
    const payload = JSON.stringify(onPay.mock.calls[0][0]);
    expect(onPay.mock.calls[0][0].token).toBe('ENCRYPTED');
    expect(onPay.mock.calls[0][0].bin).toBe('411111');
    expect(payload).not.toContain('4111 1111 1111 1111');
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

  it('lists provider options, shows the selected total, and sends the card BIN', async () => {
    window.PagSeguro = { encryptCard: vi.fn() };
    const { container, onPay, rerender } = setup();
    await waitFor(() => expect(container.querySelectorAll('#pagseguro-installments option')).toHaveLength(3));
    expect(vi.mocked(checkoutService.installments)).toHaveBeenCalledWith({ tenantId: 7, amountCents: 1000, bin: undefined });
    expect(container.querySelector('#pagseguro-installments')!.textContent).toContain('2x de R$ 5,00 sem juros');
    expect(container.querySelector('#pagseguro-installments')!.textContent).toContain('3x de R$ 3,70 (total R$ 11,10)');
    fireEvent.change(container.querySelector('#pagseguro-installments')!, { target: { value: '3' } });
    expect(container.querySelector('p')?.textContent).toContain('Total a pagar: R$ 11,10');
    expect(container.querySelector('p')?.textContent).toContain('(juros de R$ 1,10)');
    fireEvent.change(container.querySelector('#pagseguro-card-number')!, { target: { value: '5067 7667 8388 8311' } });
    await waitFor(() => expect(vi.mocked(checkoutService.installments)).toHaveBeenCalledWith({ tenantId: 7, amountCents: 1000, bin: '506776' }));
    rerender(<PagSeguroCardForm publicKey="PK" tenantId={7} amountCents={1200} email="a@b.c" cpf="52998224725" onPay={onPay} />);
    await waitFor(() => expect(vi.mocked(checkoutService.installments)).toHaveBeenCalledWith({ tenantId: 7, amountCents: 1200, bin: '506776' }));
    window.PagSeguro = { encryptCard: vi.fn().mockReturnValue({ encryptedCard: 'ENCRYPTED' }) };
    fill(container);
    fireEvent.submit(container.querySelector('form')!);
    await waitFor(() => expect(onPay).toHaveBeenCalledWith(expect.objectContaining({ installments: 3, bin: '411111' })));
  });

  it('offers only the single payment and shows the error when the options cannot be loaded', async () => {
    vi.mocked(checkoutService.installments).mockRejectedValue({ message: 'Não foi possível consultar as parcelas' });
    window.PagSeguro = { encryptCard: vi.fn() };
    const { container } = setup();
    expect(await screen.findByText(/Não foi possível consultar as parcelas/)).toBeInTheDocument();
    const options = container.querySelectorAll('#pagseguro-installments option');
    expect(options).toHaveLength(1);
    expect(options[0].textContent).toContain('à vista');
  });
});

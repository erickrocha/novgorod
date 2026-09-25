import { useEffect, useRef, useState } from 'react';

interface CardData {
  token: string;
  paymentMethodId: string;
  issuerId?: string;
  installments: number;
}
interface CardFormInstance {
  getCardFormData(): { token: string; paymentMethodId: string; issuerId?: string; installments: string };
  unmount?: () => void;
}
declare global {
  interface Window {
    MercadoPago?: new (key: string) => { cardForm: (options: unknown) => CardFormInstance };
  }
}

const loadScript = () => new Promise<void>((resolve, reject) => {
  if (window.MercadoPago) { resolve(); return; }
  const script = document.createElement('script');
  script.src = 'https://sdk.mercadopago.com/js/v2';
  script.onload = () => resolve();
  script.onerror = () => reject(new Error('Não foi possível carregar o formulário do Mercado Pago.'));
  document.head.appendChild(script);
});

export default function MercadoPagoCardForm({ publicKey, amountCents, email, cpf, onPay }:
  { publicKey: string; amountCents: number; email: string; cpf: string; onPay: (data: CardData) => Promise<void> }) {
  const [error, setError] = useState('');
  const [ready, setReady] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const onPayRef = useRef(onPay);
  onPayRef.current = onPay;

  useEffect(() => {
    let mounted = true;
    let form: CardFormInstance | undefined;
    setReady(false);
    loadScript().then(() => {
      if (!mounted || !window.MercadoPago) return;
      const mp = new window.MercadoPago(publicKey);
      form = mp.cardForm({
        amount: (amountCents / 100).toFixed(2), iframe: true,
        form: {
          id: 'form-checkout',
          cardNumber: { id: 'form-checkout__cardNumber', placeholder: 'Número do cartão' },
          expirationDate: { id: 'form-checkout__expirationDate', placeholder: 'MM/AA' },
          securityCode: { id: 'form-checkout__securityCode', placeholder: 'Código de segurança' },
          cardholderName: { id: 'form-checkout__cardholderName', placeholder: 'Nome no cartão' },
          issuer: { id: 'form-checkout__issuer', placeholder: 'Banco emissor' },
          installments: { id: 'form-checkout__installments', placeholder: 'Parcelas' },
          identificationType: { id: 'form-checkout__identificationType' },
          identificationNumber: { id: 'form-checkout__identificationNumber' },
          cardholderEmail: { id: 'form-checkout__cardholderEmail' },
        },
        callbacks: {
          onFormMounted: (mountError?: Error) => {
            if (mountError) setError('Não foi possível abrir o formulário do cartão.');
            else setReady(true);
          },
          onSubmit: async (event: Event) => {
            event.preventDefault();
            if (!form) return;
            const data = form.getCardFormData();
            if (!data.token || !data.paymentMethodId || !Number(data.installments)) {
              setError('Confira os dados do cartão e as parcelas.'); return;
            }
            setSubmitting(true); setError('');
            try { await onPayRef.current({ token: data.token, paymentMethodId: data.paymentMethodId,
              issuerId: data.issuerId, installments: Number(data.installments) }); }
            catch (e) { setError(e instanceof Error ? e.message : 'Não foi possível enviar o pagamento.'); }
            finally { if (mounted) setSubmitting(false); }
          },
        },
      });
    }).catch((e: Error) => { if (mounted) setError(e.message); });
    return () => { mounted = false; form?.unmount?.(); };
  }, [publicKey, amountCents, email, cpf]);

  const field = 'min-h-11 rounded-xl border border-slate-200 bg-white p-2.5';
  return <form id="form-checkout" className="space-y-3" onSubmit={e => e.preventDefault()}>
    <label className="block text-sm font-semibold">Número do cartão<div id="form-checkout__cardNumber" className={field} /></label>
    <div className="grid grid-cols-2 gap-3">
      <label className="block text-sm font-semibold">Validade<div id="form-checkout__expirationDate" className={field} /></label>
      <label className="block text-sm font-semibold">Código de segurança<div id="form-checkout__securityCode" className={field} /></label>
    </div>
    <label className="block text-sm font-semibold">Titular do cartão<input id="form-checkout__cardholderName" className={`${field} w-full`} required /></label>
    <label className="block text-sm font-semibold">Banco emissor<select id="form-checkout__issuer" className={`${field} w-full`} /></label>
    <label className="block text-sm font-semibold">Parcelas e custo de financiamento<select id="form-checkout__installments" className={`${field} w-full`} /></label>
    <select id="form-checkout__identificationType" defaultValue="CPF" className="sr-only"><option value="CPF">CPF</option></select>
    <input id="form-checkout__identificationNumber" type="hidden" value={cpf} readOnly />
    <input id="form-checkout__cardholderEmail" type="hidden" value={email} readOnly />
    {error && <p role="alert" className="text-rose-700 text-sm">{error}</p>}
    <button id="form-checkout__submit" type="submit" disabled={!ready || submitting}
      className="w-full rounded-xl bg-amber-600 px-5 py-3 font-bold text-white disabled:opacity-50">
      {submitting ? 'Enviando pagamento…' : 'Pagar com cartão de crédito'}
    </button>
  </form>;
}

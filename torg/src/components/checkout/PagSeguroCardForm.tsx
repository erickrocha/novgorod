import { useEffect, useRef, useState } from 'react';
import checkoutService, { type InstallmentOption } from '../../services/checkoutService';

interface CardData {
  token: string;
  paymentMethodId: string;
  issuerId?: string;
  installments: number;
  /** First digits of the card; kremlin prices the installments with it. */
  bin?: string;
}

interface EncryptCardOptions {
  publicKey: string;
  holder: string;
  number: string;
  expMonth: string;
  expYear: string;
  securityCode: string;
}

interface PagSeguroSdk {
  encryptCard: (options: EncryptCardOptions) => { encryptedCard?: string; hasErrors?: boolean; errors?: any[] };
}

declare global {
  interface Window {
    PagSeguro?: PagSeguroSdk;
  }
}

const SDK_ERROR = 'Não foi possível carregar o ambiente seguro de pagamento. Recarregue a página e tente novamente.';

// The card is only ever sent encrypted by the PagSeguro SDK: if the SDK cannot load, the form stays disabled.
const loadPagSeguroScript = () =>
  new Promise<void>((resolve, reject) => {
    if (window.PagSeguro) {
      resolve();
      return;
    }
    const script = document.createElement('script');
    script.src = 'https://assets.pagseguro.com.br/checkout-sdk-js/rc/dist/browser/pagseguro.min.js';
    script.onload = () => (window.PagSeguro ? resolve() : reject(new Error(SDK_ERROR)));
    script.onerror = () => reject(new Error(SDK_ERROR));
    document.head.appendChild(script);
  });

function detectBrand(number: string): string {
  const clean = number.replace(/\D/g, '');
  if (/^4/.test(clean)) return 'visa';
  if (/^(5[1-5]|2[2-7])/.test(clean)) return 'mastercard';
  if (/^(4011|4389|4514|4576|5041|5067|5090|6277|6362|6363)/.test(clean)) return 'elo';
  if (/^3[47]/.test(clean)) return 'amex';
  if (/^(606282|3841)/.test(clean)) return 'hipercard';
  return 'credit_card';
}

export default function PagSeguroCardForm({
  publicKey,
  tenantId,
  amountCents,
  email,
  cpf,
  onPay,
}: {
  publicKey: string;
  tenantId: number;
  amountCents: number;
  email: string;
  cpf: string;
  onPay: (data: CardData) => Promise<void>;
}) {
  const [error, setError] = useState('');
  const [ready, setReady] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const [cardNumber, setCardNumber] = useState('');
  const [cardholderName, setCardholderName] = useState('');
  const [expiry, setExpiry] = useState('');
  const [cvv, setCvv] = useState('');
  const [installments, setInstallments] = useState(1);
  const [options, setOptions] = useState<InstallmentOption[]>([]);
  const [optionsError, setOptionsError] = useState('');

  // The options come from the provider for this amount (and the card brand once the BIN is typed).
  const bin = cardNumber.replace(/\D/g, '').slice(0, 6);
  const lookupBin = bin.length === 6 ? bin : undefined;
  const selected = options.find(o => o.installments === installments);
  useEffect(() => {
    let active = true;
    const timer = window.setTimeout(() => {
      checkoutService.installments({ tenantId, amountCents, bin: lookupBin })
        .then(found => {
          if (!active) return;
          setOptions(found);
          setOptionsError('');
          setInstallments(current => (found.some(o => o.installments === current) ? current : 1));
        })
        .catch(e => {
          if (!active) return;
          setOptions([]);
          setOptionsError((e as { message?: string })?.message || 'Não foi possível carregar as parcelas.');
          setInstallments(1);
        });
    }, 250);
    return () => { active = false; window.clearTimeout(timer); };
  }, [tenantId, amountCents, lookupBin]);

  const onPayRef = useRef(onPay);
  onPayRef.current = onPay;

  useEffect(() => {
    let mounted = true;
    loadPagSeguroScript()
      .then(() => {
        if (mounted) setReady(true);
      })
      .catch(() => {
        if (mounted) setError(SDK_ERROR);
      });
    return () => {
      mounted = false;
    };
  }, []);

  const handleCardNumberChange = (val: string) => {
    const raw = val.replace(/\D/g, '').slice(0, 16);
    const formatted = raw.replace(/(\d{4})(?=\d)/g, '$1 ');
    setCardNumber(formatted);
  };

  const handleExpiryChange = (val: string) => {
    const raw = val.replace(/\D/g, '').slice(0, 4);
    if (raw.length >= 3) {
      setExpiry(`${raw.slice(0, 2)}/${raw.slice(2)}`);
    } else {
      setExpiry(raw);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    const rawNumber = cardNumber.replace(/\D/g, '');
    const cleanExpiry = expiry.replace(/\D/g, '');
    const cleanCvv = cvv.replace(/\D/g, '');

    if (rawNumber.length < 13 || rawNumber.length > 19) {
      setError('Número de cartão inválido.');
      return;
    }
    if (!cardholderName.trim()) {
      setError('Informe o nome impresso no cartão.');
      return;
    }
    if (cleanExpiry.length !== 4) {
      setError('Validade inválida (MM/AA).');
      return;
    }
    if (cleanCvv.length < 3 || cleanCvv.length > 4) {
      setError('Código de segurança (CVV) inválido.');
      return;
    }

    const expMonth = cleanExpiry.slice(0, 2);
    const expYear = `20${cleanExpiry.slice(2)}`;
    const brand = detectBrand(rawNumber);

    setSubmitting(true);
    try {
      const res = window.PagSeguro?.encryptCard({
        publicKey,
        holder: cardholderName.trim(),
        number: rawNumber,
        expMonth,
        expYear,
        securityCode: cleanCvv,
      });
      if (!res?.encryptedCard || res.hasErrors) {
        setError(res?.hasErrors ? 'Dados do cartão inválidos. Confira número, validade e CVV.' : SDK_ERROR);
        return;
      }
      const token = res.encryptedCard;

      await onPayRef.current({
        token,
        paymentMethodId: brand,
        installments,
        bin: lookupBin,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Não foi possível processar o pagamento com o PagSeguro.');
    } finally {
      setSubmitting(false);
    }
  };

  const field =
    'min-h-11 w-full rounded-xl border border-slate-200 bg-white p-2.5 text-slate-800 placeholder-slate-400 focus:border-amber-600 focus:outline-none';

  return (
    <form id="form-pagseguro" className="space-y-3" onSubmit={handleSubmit}>
      <div className="flex items-center gap-2 mb-1">
        <span className="text-xs font-semibold px-2 py-0.5 rounded bg-emerald-100 text-emerald-800">
          PagBank / PagSeguro
        </span>
      </div>

      <label className="block text-sm font-semibold text-slate-700">
        Número do cartão
        <input
          id="pagseguro-card-number"
          type="text"
          inputMode="numeric"
          placeholder="0000 0000 0000 0000"
          value={cardNumber}
          onChange={e => handleCardNumberChange(e.target.value)}
          className={field}
          required
        />
      </label>

      <div className="grid grid-cols-2 gap-3">
        <label className="block text-sm font-semibold text-slate-700">
          Validade
          <input
            id="pagseguro-card-expiry"
            type="text"
            inputMode="numeric"
            placeholder="MM/AA"
            value={expiry}
            onChange={e => handleExpiryChange(e.target.value)}
            className={field}
            required
          />
        </label>
        <label className="block text-sm font-semibold text-slate-700">
          CVV
          <input
            id="pagseguro-card-cvv"
            type="password"
            inputMode="numeric"
            placeholder="123"
            maxLength={4}
            value={cvv}
            onChange={e => setCvv(e.target.value.replace(/\D/g, ''))}
            className={field}
            required
          />
        </label>
      </div>

      <label className="block text-sm font-semibold text-slate-700">
        Titular do cartão
        <input
          id="pagseguro-card-holder"
          type="text"
          placeholder="Nome como no cartão"
          value={cardholderName}
          onChange={e => setCardholderName(e.target.value.toUpperCase())}
          className={field}
          required
        />
      </label>

      <label className="block text-sm font-semibold text-slate-700">
        Parcelas
        <select
          id="pagseguro-installments"
          value={installments}
          onChange={e => setInstallments(Number(e.target.value))}
          className={field}
        >
          {options.length === 0 && <option value={1}>1x de R$ {(amountCents / 100).toFixed(2).replace('.', ',')} (à vista)</option>}
          {options.map(o => (
            <option key={o.installments} value={o.installments}>
              {o.installments}x de R$ {(o.installmentCents / 100).toFixed(2).replace('.', ',')}
              {o.interestFree ? ' sem juros' : ` (total R$ ${(o.totalCents / 100).toFixed(2).replace('.', ',')})`}
            </option>
          ))}
        </select>
        {selected && (
          <span id="pagseguro-total" className="block mt-1 text-xs font-medium text-slate-600">
            Total a pagar: R$ {(selected.totalCents / 100).toFixed(2).replace('.', ',')}
            {selected.totalCents > amountCents && ` (juros de R$ ${((selected.totalCents - amountCents) / 100).toFixed(2).replace('.', ',')})`}
          </span>
        )}
        {optionsError && <span role="alert" className="block mt-1 text-xs font-medium text-rose-700">{optionsError} Apenas à vista está disponível.</span>}
      </label>

      <input type="hidden" id="pagseguro-email" value={email} readOnly />
      <input type="hidden" id="pagseguro-cpf" value={cpf} readOnly />

      {error && (
        <p role="alert" className="text-rose-700 text-sm font-medium">
          {error}
        </p>
      )}

      <button
        id="pagseguro-submit"
        type="submit"
        disabled={!ready || submitting}
        className="w-full rounded-xl bg-amber-600 px-5 py-3 font-bold text-white hover:bg-amber-700 disabled:opacity-50 transition"
      >
        {submitting ? 'Enviando pagamento PagSeguro…' : 'Pagar com PagSeguro'}
      </button>
    </form>
  );
}

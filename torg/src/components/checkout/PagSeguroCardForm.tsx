import { useEffect, useRef, useState } from 'react';

interface CardData {
  token: string;
  paymentMethodId: string;
  issuerId?: string;
  installments: number;
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

const loadPagSeguroScript = () =>
  new Promise<void>((resolve) => {
    if (window.PagSeguro) {
      resolve();
      return;
    }
    const script = document.createElement('script');
    script.src = 'https://assets.pagseguro.com.br/checkout-sdk-js/rc/dist/pagseguro.min.js';
    script.onload = () => resolve();
    script.onerror = () => {
      // If CDN is blocked or unavailable in local testing, continue gracefully
      resolve();
    };
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
  amountCents,
  email,
  cpf,
  onPay,
}: {
  publicKey: string;
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

  const onPayRef = useRef(onPay);
  onPayRef.current = onPay;

  useEffect(() => {
    let mounted = true;
    loadPagSeguroScript()
      .then(() => {
        if (mounted) setReady(true);
      })
      .catch(() => {
        if (mounted) setReady(true);
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
      let token = '';

      if (window.PagSeguro && typeof window.PagSeguro.encryptCard === 'function' && publicKey) {
        try {
          const res = window.PagSeguro.encryptCard({
            publicKey,
            holder: cardholderName.trim(),
            number: rawNumber,
            expMonth,
            expYear,
            securityCode: cleanCvv,
          });
          if (res?.encryptedCard) {
            token = res.encryptedCard;
          }
        } catch {
          // fallback below
        }
      }

      if (!token) {
        // Fallback card payload for server-side / sandbox proxy
        token = JSON.stringify({
          number: rawNumber,
          holder: { name: cardholderName.trim() },
          exp_month: Number(expMonth),
          exp_year: Number(expYear),
          security_code: cleanCvv,
        });
      }

      await onPayRef.current({
        token,
        paymentMethodId: brand,
        installments,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Não foi possível processar o pagamento com o PagSeguro.');
    } finally {
      setSubmitting(false);
    }
  };

  const field =
    'min-h-11 w-full rounded-xl border border-slate-200 bg-white p-2.5 text-slate-800 placeholder-slate-400 focus:border-amber-600 focus:outline-none';

  const installmentOptions = [1, 2, 3, 4, 5, 6, 10, 12];

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
          {installmentOptions.map(n => {
            const installmentAmount = (amountCents / 100 / n).toFixed(2);
            return (
              <option key={n} value={n}>
                {n}x de R$ {installmentAmount.replace('.', ',')} sem juros
              </option>
            );
          })}
        </select>
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

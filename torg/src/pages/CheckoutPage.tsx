import { useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { selectCartItems } from '../store';
import { logout, setCustomerSession, type CustomerProfile } from '../store/slices/authSlice';
import { removeFromCart } from '../store/slices/cartSlice';
import authService from '../services/authService';
import catalogService from '../services/catalogService';
import checkoutService, { type CheckoutQuote, type DeliveryAddress, type Purchase, type PaymentState } from '../services/checkoutService';
import type { ProductSku } from '../types/product';
import { CheckoutAuth } from '../components/checkout/CheckoutAuth';
import MercadoPagoCardForm from '../components/checkout/MercadoPagoCardForm';
import { Breadcrumb } from '../components/common/Breadcrumb';
import { CreditCard, Truck, CheckCircle2, AlertTriangle, ShieldCheck } from 'lucide-react';

const money = (cents: number) => new Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' }).format(cents / 100);

export const isCompleteAddress = (a: { recipient?: string; addressLine1?: string; locality?: string; administrativeArea?: string; postalCode?: string } | null | undefined): boolean =>
  Boolean(
    a?.recipient?.trim() &&
    a?.addressLine1?.trim() &&
    a?.locality?.trim() &&
    a?.administrativeArea && /^[A-Za-z]{2}$/.test(a.administrativeArea.trim()) &&
    a?.postalCode && /^\d{5}-?\d{3}$|^\d{8}$/.test(a.postalCode.trim())
  );

const newAddress = (): DeliveryAddress => ({
  recipient: '',
  addressLine1: '',
  addressLine2: '',
  locality: '',
  administrativeArea: '',
  postalCode: '',
  countryCode: 'BR',
});

const errorText = (error: unknown) =>
  error && typeof error === 'object' && 'message' in error ? String(error.message) : 'Não foi possível concluir a operação.';

const box = 'rounded-3xl border border-slate-200 bg-white p-6 sm:p-7 shadow-xs space-y-4';
const input = 'mt-1 w-full rounded-xl border border-slate-200 p-3 text-sm focus:border-amber-500 focus:outline-none';

export const CheckoutPage = () => {
  const dispatch = useAppDispatch();
  const items = useAppSelector(selectCartItems);
  const { token } = useAppSelector(state => state.auth);
  const [profile, setProfile] = useState<CustomerProfile | null>(null);
  const [loading, setLoading] = useState(Boolean(token));
  const [email, setEmail] = useState('');
  const [phone, setPhone] = useState('');
  const [cpf, setCpf] = useState('');
  const [manual, setManual] = useState(true);
  const [addressId, setAddressId] = useState<number | null>(null);
  const [address, setAddress] = useState<DeliveryAddress>(newAddress);
  const [skuOptions, setSkuOptions] = useState<Record<string, ProductSku[]>>({});
  const [skuSelection, setSkuSelection] = useState<Record<string, number>>({});
  const [coupons, setCoupons] = useState<Record<number, string>>({});
  const [quote, setQuote] = useState<CheckoutQuote | null>(null);
  const [purchase, setPurchase] = useState<Purchase | null>(null);
  const [publicKey, setPublicKey] = useState('');
  const [payment, setPayment] = useState<PaymentState | null>(null);
  const [paymentMethod, setPaymentMethod] = useState<'credit_card'>('credit_card');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  const completeSavedAddresses = useMemo(() => {
    return (profile?.addresses || []).filter(isCompleteAddress);
  }, [profile?.addresses]);

  useEffect(() => {
    if (!token) { setProfile(null); setLoading(false); return; }
    let active = true;
    setLoading(true);
    authService.me(token).then(customer => {
      if (!active) return;
      setProfile(customer);
      dispatch(setCustomerSession({ customer, token }));
      setEmail(customer.email);
      setPhone(customer.phone || '');
      const completeList = (customer.addresses || []).filter(isCompleteAddress);
      if (completeList.length > 0) {
        const def = completeList.find(a => a.isDefault) || completeList[0];
        setAddressId(Number(def.id));
        setManual(false);
      } else {
        setAddressId(null);
        setManual(true);
      }
    }).catch(e => {
      if (active) {
        if (e?.status === 401 || e?.status === 403) dispatch(logout());
        else setError(errorText(e));
      }
    }).finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [token, dispatch]);

  useEffect(() => {
    let active = true;
    Promise.all(items.map(item => catalogService.getProductBySlug(item.product.slug).then(product => {
      if (!product) throw new Error(`Produto ${item.product.name} indisponível.`);
      return { id: item.id, skus: (product.skus || []).filter(s => s.active), variantId: item.variant?.id };
    }))).then(rows => {
      if (!active) return;
      const options: Record<string, ProductSku[]> = {};
      const selection: Record<string, number> = {};
      for (const row of rows) {
        options[row.id] = row.skus;
        const exact = row.skus.find(s => s.id === Number(row.variantId));
        if (exact) selection[row.id] = exact.id;
        else if (row.skus.length === 1) selection[row.id] = row.skus[0].id;
      }
      setSkuOptions(options);
      setSkuSelection(selection);
    }).catch(e => { if (active) setError(errorText(e)); });
    return () => { active = false; };
  }, [items]);

  useEffect(() => {
    if (!token) return;
    const saved = sessionStorage.getItem('torg_pending_purchase');
    if (!saved) return;
    try {
      const restored = JSON.parse(saved) as Purchase;
      setPurchase(restored);
      checkoutService.status(restored.id).then(setPayment).catch(() => {});
      checkoutService.paymentConfig().then(config => setPublicKey(config.publicKey)).catch(() => {});
    } catch {
      sessionStorage.removeItem('torg_pending_purchase');
    }
  }, [token]);

  useEffect(() => {
    if (!purchase || !payment || !['pending', 'authorized'].includes(payment.status)) return;
    const timer = window.setInterval(() => {
      checkoutService.status(purchase.id).then(setPayment).catch(() => {});
    }, 5000);
    return () => window.clearInterval(timer);
  }, [purchase, payment]);

  useEffect(() => {
    if (payment?.status !== 'captured' || !purchase) return;
    for (const item of items) {
      dispatch(removeFromCart(item.id));
    }
    sessionStorage.removeItem('torg_pending_purchase');
  }, [payment?.status, purchase, dispatch, items]);

  const addressReady = manual
    ? Boolean(
        address.recipient.trim() &&
        address.addressLine1.trim() &&
        address.locality.trim() &&
        /^[A-Za-z]{2}$/.test(address.administrativeArea.trim()) &&
        /^\d{5}-?\d{3}$|^\d{8}$/.test(address.postalCode.trim())
      )
    : Boolean(addressId && completeSavedAddresses.some(a => Number(a.id) === addressId));

  const skusReady = items.length > 0 && items.every(item => skuOptions[item.id]?.some(s => s.id === skuSelection[item.id]));
  const sellerIds = Array.from(new Set(items.map(item => item.product.seller?.id).filter((id): id is number => Boolean(id))));
  const sellerName = (id: number) => items.find(item => item.product.seller?.id === id)?.product.seller?.businessName || `Vendedor ${id}`;
  const resetQuote = () => {
    setQuote(null);
    setPurchase(null);
    setPayment(null);
  };

  async function completeCpf() {
    setBusy(true);
    setError('');
    try {
      const updated = await authService.completeCpf(cpf);
      setProfile(updated);
      if (token) dispatch(setCustomerSession({ customer: updated, token }));
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function calculate() {
    if (!profile?.cpf || !addressReady || !skusReady || !email.includes('@') || phone.trim().length < 8) {
      setError('Confira CPF, contato, endereço e variantes dos produtos.');
      return;
    }
    setBusy(true);
    setError('');
    resetQuote();
    try {
      const result = await checkoutService.quote({
        items: items.map(item => ({ skuId: skuSelection[item.id], quantity: item.quantity })),
        ...(manual
          ? {
              shippingAddress: {
                ...address,
                administrativeArea: address.administrativeArea.trim().toUpperCase(),
                postalCode: address.postalCode.replace(/\D/g, ''),
              },
            }
          : { addressId: addressId! }),
        coupons: Object.entries(coupons)
          .filter(([, code]) => code.trim())
          .map(([tenantId, code]) => ({ tenantId: Number(tenantId), code: code.trim() })),
      });
      setQuote(result);
      if (result.totalCents > 0) {
        checkoutService.paymentConfig().then(cfg => setPublicKey(cfg.publicKey)).catch(() => {});
      }
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function pay(data: { token: string; paymentMethodId: string; issuerId?: string; installments: number }) {
    if (!quote) return;
    setBusy(true);
    setError('');
    try {
      let currentPurchase = purchase;
      if (!currentPurchase) {
        currentPurchase = await checkoutService.createPurchase(quote.id, email, phone);
        setPurchase(currentPurchase);
        sessionStorage.setItem('torg_pending_purchase', JSON.stringify(currentPurchase));
      }
      const paymentState = await checkoutService.submit(currentPurchase.id, data);
      setPayment(paymentState);
    } catch (e) {
      setError(errorText(e));
      throw e;
    } finally {
      setBusy(false);
    }
  }

  async function completeZeroTotal() {
    if (!quote || quote.totalCents !== 0) return;
    setBusy(true);
    setError('');
    try {
      const created = await checkoutService.createPurchase(quote.id, email, phone);
      setPurchase(created);
      setPayment({ purchaseId: created.id, status: 'captured' });
    } catch (e) {
      setError(errorText(e));
      resetQuote();
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">
      <Breadcrumb items={[{ label: 'Carrinho', href: '/carrinho' }, { label: 'Checkout' }]} />
      <h1 className="text-3xl font-extrabold text-slate-900 tracking-tight">Finalização de compra</h1>

      {error && (
        <div role="alert" className="rounded-2xl bg-rose-50 border border-rose-200 p-4 text-rose-800 flex items-start gap-3">
          <AlertTriangle className="h-5 w-5 text-rose-600 mt-0.5 shrink-0" />
          <p className="text-sm font-medium">{error}</p>
        </div>
      )}

      {loading ? (
        <div className="p-12 text-center text-slate-500 font-medium">Verificando sua sessão…</div>
      ) : !profile ? (
        <CheckoutAuth />
      ) : payment?.status === 'captured' ? (
        <section className={box}>
          <div className="flex items-center gap-3 text-emerald-700">
            <CheckCircle2 className="h-8 w-8 shrink-0" />
            <h2 className="text-2xl font-bold">Pagamento confirmado</h2>
          </div>
          <p className="text-slate-700">
            Compra #{purchase?.id} confirmada com sucesso. Pedidos:{' '}
            <strong className="text-slate-900">{purchase?.orders.map(order => order.number).join(', ')}</strong>.
          </p>
          <p className="text-sm text-slate-500">Uma cópia dos detalhes do pedido foi enviada para o seu e-mail.</p>
          <div className="pt-2">
            <Link className="inline-block rounded-xl bg-amber-600 px-6 py-3 font-bold text-white hover:bg-amber-700" to="/">
              Voltar ao catálogo
            </Link>
          </div>
        </section>
      ) : (
        <div className="grid gap-8 lg:grid-cols-[2fr_1fr]">
          <div className="space-y-6">
            {/* Step 1: Customer Details */}
            <section className={box}>
              <h2 className="text-xl font-bold text-slate-900 flex items-center gap-2">
                <span className="flex h-7 w-7 items-center justify-center rounded-full bg-amber-600 text-white text-xs font-bold">1</span>
                Seus dados
              </h2>
              <div className="grid gap-3 sm:grid-cols-2">
                <label className="text-xs font-semibold text-slate-700">
                  Nome completo (cadastrado)
                  <input readOnly value={profile.name} className={`${input} bg-slate-50 text-slate-600 cursor-not-allowed`} />
                </label>
                <label className="text-xs font-semibold text-slate-700">
                  CPF (cadastrado)
                  <input readOnly value={profile.cpf || 'Não informado'} className={`${input} bg-slate-50 text-slate-600 cursor-not-allowed`} />
                </label>
                <label className="text-xs font-semibold text-slate-700">
                  E-mail para este pedido
                  <input
                    type="email"
                    value={email}
                    disabled={!!purchase}
                    onChange={e => { setEmail(e.target.value); resetQuote(); }}
                    className={input}
                  />
                </label>
                <label className="text-xs font-semibold text-slate-700">
                  Telefone para este pedido
                  <input
                    type="tel"
                    value={phone}
                    disabled={!!purchase}
                    onChange={e => { setPhone(e.target.value); resetQuote(); }}
                    className={input}
                  />
                </label>
              </div>

              {!profile.cpf && (
                <div className="rounded-2xl bg-amber-50 border border-amber-200 p-4 space-y-3">
                  <p className="text-sm font-semibold text-amber-900">
                    Informe seu CPF para continuar. Ele será salvo uma única vez e associado ao seu cadastro.
                  </p>
                  <div className="flex gap-2">
                    <input
                      aria-label="CPF para completar cadastro"
                      placeholder="000.000.000-00"
                      value={cpf}
                      onChange={e => setCpf(e.target.value)}
                      className="rounded-xl border border-amber-300 p-2.5 text-sm flex-1"
                    />
                    <button
                      type="button"
                      disabled={busy || !cpf.trim()}
                      onClick={completeCpf}
                      className="rounded-xl bg-amber-600 px-4 py-2.5 text-sm font-bold text-white hover:bg-amber-700 disabled:opacity-50"
                    >
                      Salvar CPF
                    </button>
                  </div>
                </div>
              )}
            </section>

            {/* Step 2: Delivery */}
            {!purchase && (
              <section className={box}>
                <h2 className="text-xl font-bold text-slate-900 flex items-center gap-2">
                  <span className="flex h-7 w-7 items-center justify-center rounded-full bg-amber-600 text-white text-xs font-bold">2</span>
                  <Truck className="h-5 w-5 text-amber-600" />
                  Entrega
                </h2>

                {completeSavedAddresses.length > 0 ? (
                  <fieldset className="space-y-3">
                    <legend className="text-sm font-semibold text-slate-700 mb-2">
                      Deseja usar um endereço salvo para entrega?
                    </legend>
                    {completeSavedAddresses.map(saved => (
                      <label
                        key={saved.id}
                        className={`block rounded-2xl border p-4 cursor-pointer transition-colors ${
                          !manual && addressId === Number(saved.id)
                            ? 'border-amber-600 bg-amber-50/40 ring-1 ring-amber-600'
                            : 'border-slate-200 hover:border-slate-300'
                        }`}
                      >
                        <div className="flex items-start gap-3">
                          <input
                            type="radio"
                            name="delivery-mode"
                            checked={!manual && addressId === Number(saved.id)}
                            onChange={() => { setManual(false); setAddressId(Number(saved.id)); resetQuote(); }}
                            className="mt-1 text-amber-600"
                          />
                          <div>
                            <span className="font-bold text-slate-900">{saved.label || saved.recipient}</span>
                            {saved.isDefault && (
                              <span className="ml-2 text-xs font-semibold uppercase tracking-wider text-amber-700 bg-amber-100 px-2 py-0.5 rounded-full">
                                Padrão
                              </span>
                            )}
                            <p className="text-sm text-slate-600 mt-0.5">
                              {saved.addressLine1}{saved.addressLine2 ? `, ${saved.addressLine2}` : ''} — {saved.locality}/{saved.administrativeArea} (CEP: {saved.postalCode})
                            </p>
                          </div>
                        </div>
                      </label>
                    ))}
                    <label
                      className={`block rounded-2xl border p-4 cursor-pointer transition-colors ${
                        manual
                          ? 'border-amber-600 bg-amber-50/40 ring-1 ring-amber-600'
                          : 'border-slate-200 hover:border-slate-300'
                      }`}
                    >
                      <div className="flex items-center gap-3">
                        <input
                          type="radio"
                          name="delivery-mode"
                          checked={manual}
                          onChange={() => { setManual(true); setAddressId(null); resetQuote(); }}
                          className="text-amber-600"
                        />
                        <span className="font-semibold text-slate-900">Outro endereço para este pedido</span>
                      </div>
                    </label>
                  </fieldset>
                ) : (
                  <p className="text-sm text-slate-600">
                    Nenhum endereço completo cadastrado. Informe os dados de entrega para este pedido:
                  </p>
                )}

                {manual && (
                  <div className="grid gap-3 sm:grid-cols-2 pt-2">
                    <label className="text-xs font-semibold text-slate-700 sm:col-span-2">
                      Destinatário
                      <input
                        value={address.recipient}
                        onChange={e => { setAddress(old => ({ ...old, recipient: e.target.value })); resetQuote(); }}
                        className={input}
                        placeholder="Nome de quem receberá o pedido"
                      />
                    </label>
                    <label className="text-xs font-semibold text-slate-700">
                      Rua e número
                      <input
                        value={address.addressLine1}
                        onChange={e => { setAddress(old => ({ ...old, addressLine1: e.target.value })); resetQuote(); }}
                        className={input}
                        placeholder="Ex: Av. Paulista, 1000"
                      />
                    </label>
                    <label className="text-xs font-semibold text-slate-700">
                      Complemento (opcional)
                      <input
                        value={address.addressLine2 || ''}
                        onChange={e => { setAddress(old => ({ ...old, addressLine2: e.target.value })); resetQuote(); }}
                        className={input}
                        placeholder="Apto, Bloco, etc."
                      />
                    </label>
                    <label className="text-xs font-semibold text-slate-700">
                      Cidade
                      <input
                        value={address.locality}
                        onChange={e => { setAddress(old => ({ ...old, locality: e.target.value })); resetQuote(); }}
                        className={input}
                        placeholder="Ex: São Paulo"
                      />
                    </label>
                    <label className="text-xs font-semibold text-slate-700">
                      UF (Estado - 2 letras)
                      <input
                        value={address.administrativeArea}
                        maxLength={2}
                        onChange={e => { setAddress(old => ({ ...old, administrativeArea: e.target.value.toUpperCase() })); resetQuote(); }}
                        className={input}
                        placeholder="Ex: SP"
                      />
                    </label>
                    <label className="text-xs font-semibold text-slate-700">
                      CEP
                      <input
                        value={address.postalCode}
                        maxLength={9}
                        onChange={e => { setAddress(old => ({ ...old, postalCode: e.target.value })); resetQuote(); }}
                        className={input}
                        placeholder="00000-000"
                      />
                    </label>
                    <label className="text-xs font-semibold text-slate-700">
                      País
                      <input
                        value="Brasil (BR)"
                        readOnly
                        className={`${input} bg-slate-50 text-slate-500 cursor-not-allowed`}
                      />
                    </label>
                  </div>
                )}
              </section>
            )}

            {/* Step 3: Products and Coupons */}
            {!purchase && (
              <section className={box}>
                <h2 className="text-xl font-bold text-slate-900 flex items-center gap-2">
                  <span className="flex h-7 w-7 items-center justify-center rounded-full bg-amber-600 text-white text-xs font-bold">3</span>
                  Produtos e cupons
                </h2>

                <div className="divide-y divide-slate-100">
                  {items.map(item => (
                    <div key={item.id} className="py-3 flex flex-wrap items-center justify-between gap-3">
                      <div>
                        <strong className="text-sm font-semibold text-slate-900">{item.product.name}</strong>
                        <p className="text-xs text-slate-500">Quantidade: {item.quantity}</p>
                      </div>
                      {skuOptions[item.id]?.length > 1 ? (
                        <select
                          aria-label={`Variante de ${item.product.name}`}
                          value={skuSelection[item.id] || ''}
                          onChange={e => {
                            setSkuSelection(old => ({ ...old, [item.id]: Number(e.target.value) }));
                            resetQuote();
                          }}
                          className="rounded-xl border border-slate-200 p-2 text-xs font-medium"
                        >
                          <option value="">Selecione uma variante</option>
                          {skuOptions[item.id].map(sku => (
                            <option key={sku.id} value={sku.id}>
                              {sku.variantKey || sku.code} — {money(sku.priceCents)}
                            </option>
                          ))}
                        </select>
                      ) : skuOptions[item.id]?.length === 0 ? (
                        <span className="text-xs font-semibold text-rose-700">Produto indisponível</span>
                      ) : null}
                    </div>
                  ))}
                </div>

                <div className="pt-2 space-y-3">
                  {sellerIds.map(id => (
                    <label key={id} className="block text-xs font-semibold text-slate-700">
                      Cupom de desconto — {sellerName(id)}
                      <input
                        value={coupons[id] || ''}
                        onChange={e => {
                          setCoupons(old => ({ ...old, [id]: e.target.value.toUpperCase() }));
                          resetQuote();
                        }}
                        className={input}
                        placeholder="Código do cupom (ex: PROMO10)"
                      />
                    </label>
                  ))}
                </div>

                <div className="pt-2">
                  <button
                    type="button"
                    disabled={busy || !addressReady || !skusReady || !profile.cpf}
                    onClick={calculate}
                    className="w-full sm:w-auto rounded-xl bg-slate-900 px-6 py-3 font-bold text-white hover:bg-slate-800 disabled:opacity-50 transition-colors"
                  >
                    {busy ? 'Calculando frete e cupons…' : 'Calcular total com frete'}
                  </button>
                </div>
              </section>
            )}

            {/* Step 4: Payment */}
            {quote && (
              <section className={box}>
                <h2 className="text-xl font-bold text-slate-900 flex items-center gap-2">
                  <span className="flex h-7 w-7 items-center justify-center rounded-full bg-amber-600 text-white text-xs font-bold">4</span>
                  <CreditCard className="h-5 w-5 text-amber-600" />
                  Pagamento
                </h2>

                <fieldset className="space-y-3">
                  <legend className="text-sm font-semibold text-slate-700 mb-2">Forma de pagamento</legend>
                  <label className="flex items-center gap-3 rounded-2xl border border-amber-600 bg-amber-50/40 p-4 font-bold text-slate-900 cursor-pointer">
                    <input
                      type="radio"
                      name="payment-selection"
                      value="credit_card"
                      checked={paymentMethod === 'credit_card'}
                      onChange={() => setPaymentMethod('credit_card')}
                      className="text-amber-600"
                    />
                    <span>Cartão de crédito</span>
                  </label>
                </fieldset>

                {payment?.status === 'pending' || payment?.status === 'authorized' ? (
                  <div className="rounded-2xl bg-amber-50 border border-amber-200 p-6 text-center space-y-2">
                    <p className="text-lg font-bold text-amber-900">Pagamento em processamento</p>
                    <p className="text-sm text-slate-600">Aguardando confirmação do Mercado Pago. Esta página atualizará automaticamente.</p>
                  </div>
                ) : quote.totalCents === 0 ? (
                  <div className="space-y-3 pt-2">
                    <p className="text-sm text-slate-600">Total da compra é R$ 0,00. Nenhum cartão de crédito necessário.</p>
                    <button
                      type="button"
                      disabled={busy}
                      onClick={completeZeroTotal}
                      className="w-full rounded-xl bg-amber-600 px-5 py-3 font-bold text-white hover:bg-amber-700 disabled:opacity-50"
                    >
                      {busy ? 'Concluindo…' : 'Concluir pedido gratuito'}
                    </button>
                  </div>
                ) : (
                  <div className="space-y-4 pt-2">
                    {payment?.status === 'failed' && (
                      <div className="rounded-xl bg-rose-50 border border-rose-200 p-4 text-sm text-rose-800 font-medium">
                        Pagamento recusado. Confira os dados e tente outro cartão.
                      </div>
                    )}
                    {publicKey ? (
                      <MercadoPagoCardForm
                        key={`${quote.totalCents}-${payment?.reference || 'new'}`}
                        publicKey={publicKey}
                        amountCents={quote.totalCents}
                        email={email}
                        cpf={profile.cpf || ''}
                        onPay={pay}
                      />
                    ) : (
                      <p className="text-sm text-slate-500">Carregando configuração de pagamento…</p>
                    )}
                  </div>
                )}
              </section>
            )}
          </div>

          {/* Order Summary Aside */}
          <aside className={`${box} h-fit lg:sticky lg:top-24 space-y-4`}>
            <h2 className="text-xl font-bold text-slate-900">Resumo da compra</h2>

            {quote ? (
              <div className="space-y-4">
                {quote.sellers.map(seller => (
                  <div key={seller.tenantId} className="border-t border-slate-100 pt-3 space-y-1 text-sm">
                    <p className="font-bold text-slate-900">{sellerName(seller.tenantId)}</p>
                    <div className="flex justify-between text-slate-600">
                      <span>Produtos:</span>
                      <span>{money(seller.subtotalCents)}</span>
                    </div>
                    {seller.discountCents > 0 && (
                      <div className="flex justify-between text-emerald-700 font-medium">
                        <span>Desconto{seller.couponCode ? ` (${seller.couponCode})` : ''}:</span>
                        <span>−{money(seller.discountCents)}</span>
                      </div>
                    )}
                    <div className="flex justify-between text-slate-600">
                      <span>Frete:</span>
                      <span>{money(seller.shippingCents)}</span>
                    </div>
                    <div className="flex justify-between font-semibold text-slate-900 pt-1">
                      <span>Subtotal vendedor:</span>
                      <span>{money(seller.totalCents)}</span>
                    </div>
                  </div>
                ))}

                <div className="border-t-2 border-slate-200 pt-3 space-y-1">
                  <div className="flex justify-between text-sm text-slate-600">
                    <span>Subtotal de produtos:</span>
                    <span>{money(quote.subtotalCents)}</span>
                  </div>
                  {quote.discountCents > 0 && (
                    <div className="flex justify-between text-sm text-emerald-700 font-medium">
                      <span>Total em descontos:</span>
                      <span>−{money(quote.discountCents)}</span>
                    </div>
                  )}
                  <div className="flex justify-between text-sm text-slate-600">
                    <span>Total em fretes:</span>
                    <span>{money(quote.shippingCents)}</span>
                  </div>
                  <div className="flex justify-between text-xl font-extrabold text-slate-900 pt-2 border-t border-slate-100">
                    <span>Total a pagar:</span>
                    <span className="text-amber-700">{money(quote.totalCents)}</span>
                  </div>
                </div>

                <div className="flex items-center gap-2 text-xs text-slate-500 pt-2">
                  <ShieldCheck className="h-4 w-4 text-emerald-600" />
                  <span>Pagamento 100% seguro via Mercado Pago</span>
                </div>
              </div>
            ) : purchase ? (
              <div className="space-y-2 border-t border-slate-100 pt-3">
                <p className="text-sm text-slate-600">Compra #{purchase.id}</p>
                <p className="text-2xl font-extrabold text-amber-700">{money(purchase.totalCents)}</p>
              </div>
            ) : (
              <div className="border-t border-slate-100 pt-3 text-sm text-slate-500">
                Confirme seu endereço de entrega e variantes para calcular o frete e o total final.
              </div>
            )}

            <div className="border-t border-slate-100 pt-3 text-center">
              <Link to="/carrinho" className="text-sm font-semibold text-amber-700 hover:text-amber-800 underline">
                Revisar carrinho de compras
              </Link>
            </div>
          </aside>
        </div>
      )}
    </div>
  );
};

export default CheckoutPage;


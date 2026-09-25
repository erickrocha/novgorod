import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { Link } from 'react-router-dom';
import { useAppDispatch } from '../../store/hooks';
import { setCustomerSession } from '../../store/slices/authSlice';
import Button from '../common/Button';
import authService from '../../services/authService';

interface LoginFormData {
  identifier: string; // Email or CPF
  password: string;
}

const loginSchema = yup.object({
  identifier: yup.string().trim().required('E-mail ou CPF é obrigatório'),
  password: yup.string().trim().required('Senha é obrigatória'),
});

export const CheckoutAuth: React.FC = () => {
  const dispatch = useAppDispatch();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState('');

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormData>({
    resolver: yupResolver(loginSchema),
  });

  const onSubmit = async (data: LoginFormData) => {
    setIsSubmitting(true);
    setErrorMsg('');
    try {
      const response = await authService.login(data.identifier, data.password);
      if (response && response.accessToken) {
        const customer = await authService.me(response.accessToken);
        dispatch(
          setCustomerSession({
            customer,
            token: response.accessToken,
          })
        );
      }
    } catch (error: any) {
      if (error?.message) {
        setErrorMsg(error.message);
      } else {
        setErrorMsg('Erro ao fazer login. Verifique suas credenciais.');
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSubmit(onSubmit)();
    }
  };

  return (
    <div className="bg-white rounded-3xl border border-slate-200/80 p-6 sm:p-8 shadow-xs space-y-4">
      <h2 className="text-lg font-bold text-slate-900">
        Acesse sua conta para continuar
      </h2>
      <p className="text-sm text-slate-500">
        Para prosseguir com o pagamento e entrega, você precisa estar logado.
      </p>

      {errorMsg && (
        <div className="p-3 bg-red-50 text-red-700 text-sm rounded-lg">
          {errorMsg}
        </div>
      )}

      <div onKeyDown={handleKeyDown} className="space-y-4 pt-2">
        <div>
          <label className="text-xs font-semibold text-slate-700 block mb-1">
            E-mail ou CPF
          </label>
          <input
            {...register('identifier')}
            className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
              errors.identifier ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
            }`}
            placeholder="seu@email.com ou 000.000.000-00"
          />
          {errors.identifier && (
            <p className="text-xs text-rose-600 mt-1">{errors.identifier.message}</p>
          )}
        </div>

        <div>
          <label className="text-xs font-semibold text-slate-700 block mb-1">
            Senha
          </label>
          <input
            {...register('password')}
            type="password"
            className={`w-full p-2.5 rounded-xl border text-sm focus:outline-none ${
              errors.password ? 'border-rose-500' : 'border-slate-200 focus:border-amber-500'
            }`}
            placeholder="******"
          />
          {errors.password && (
            <p className="text-xs text-rose-600 mt-1">{errors.password.message}</p>
          )}
        </div>

        <Button
          type="button"
          onClick={handleSubmit(onSubmit)}
          variant="primary"
          isLoading={isSubmitting}
          className="w-full font-bold shadow-md mt-2"
        >
          Entrar
        </Button>
      </div>

      <div className="pt-4 border-t border-slate-100 text-center">
        <p className="text-sm text-slate-600">
          Ainda não tem conta?{' '}
          <Link
            to="/cadastro?redirect=/checkout"
            className="text-amber-600 font-bold hover:underline"
          >
            Cadastre-se aqui
          </Link>
        </p>
      </div>
    </div>
  );
};

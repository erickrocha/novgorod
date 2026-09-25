import React, { useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { useAppDispatch } from '../store/hooks';
import { setCustomerSession } from '../store/slices/authSlice';
import { addToast } from '../store/slices/uiSlice';
import Button from '../components/common/Button';
import authService from '../services/authService';

interface SignUpFormData {
  name: string;
  email: string;
  cpf: string;
  phone: string;
  password: string;
}

const formatCPF = (value: string) => {
  return value
    .replace(/\D/g, '')
    .replace(/(\d{3})(\d)/, '$1.$2')
    .replace(/(\d{3})(\d)/, '$1.$2')
    .replace(/(\d{3})(\d{1,2})/, '$1-$2')
    .replace(/(-\d{2})\d+?$/, '$1');
};

const formatPhone = (value: string) => {
  return value
    .replace(/\D/g, '')
    .replace(/(\d{2})(\d)/, '($1) $2')
    .replace(/(\d{4,5})(\d{4})/, '$1-$2')
    .replace(/(-\d{4})\d+?$/, '$1');
};

const signUpSchema = yup.object({
  name: yup.string().trim().required('Nome é obrigatório'),
  email: yup.string().trim().email('E-mail inválido').required('E-mail é obrigatório'),
  cpf: yup.string().trim().required('CPF é obrigatório'),
  phone: yup.string().trim().required('Telefone é obrigatório'),
  password: yup.string().trim().min(6, 'Senha deve ter no mínimo 6 caracteres').required('Senha é obrigatória'),
});

export const SignUpPage: React.FC = () => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState('');

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<SignUpFormData>({
    resolver: yupResolver(signUpSchema),
  });

  const redirectUrl = searchParams.get('redirect') || '/';

  const onSubmit = async (data: SignUpFormData) => {
    setIsSubmitting(true);
    setErrorMsg('');
    try {
      // 1. Register customer
      await authService.signup({
        name: data.name,
        email: data.email,
        password: data.password,
        cpf: data.cpf,
        phone: data.phone,
      });

      // 2. Automatically log in after registration
      const loginRes = await authService.login(data.email, data.password);

      if (loginRes && loginRes.accessToken) {
        const customer = await authService.me(loginRes.accessToken);
        dispatch(
          setCustomerSession({
            customer,
            token: loginRes.accessToken,
          })
        );
      }

      dispatch(
        addToast({
          type: 'success',
          message: 'Sua conta foi criada com sucesso!',
        })
      );
      navigate(redirectUrl);
    } catch (error: any) {
      if (error?.message) {
        setErrorMsg(error.message);
      } else {
        setErrorMsg('Erro ao criar conta. Tente novamente mais tarde.');
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="max-w-md mx-auto my-12 p-6 bg-white rounded-xl shadow-sm border border-gray-100">
      <h1 className="text-2xl font-bold text-gray-900 mb-6 text-center">Criar Conta</h1>
      
      {errorMsg && (
        <div className="mb-4 p-4 bg-red-50 text-red-700 text-sm rounded-lg">
          {errorMsg}
        </div>
      )}

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Nome Completo</label>
          <input
            {...register('name')}
            className={`w-full p-3 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-colors ${
              errors.name ? 'border-red-500' : 'border-gray-200'
            }`}
            placeholder="Seu nome completo"
          />
          {errors.name && <p className="mt-1 text-sm text-red-500">{errors.name.message}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">E-mail</label>
          <input
            {...register('email')}
            type="email"
            className={`w-full p-3 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-colors ${
              errors.email ? 'border-red-500' : 'border-gray-200'
            }`}
            placeholder="seu@email.com"
          />
          {errors.email && <p className="mt-1 text-sm text-red-500">{errors.email.message}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">CPF</label>
          <input
            {...register('cpf')}
            onChange={(e) => {
              e.target.value = formatCPF(e.target.value);
              register('cpf').onChange(e);
            }}
            className={`w-full p-3 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-colors ${
              errors.cpf ? 'border-red-500' : 'border-gray-200'
            }`}
            placeholder="000.000.000-00"
            maxLength={14}
          />
          {errors.cpf && <p className="mt-1 text-sm text-red-500">{errors.cpf.message}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Telefone</label>
          <input
            {...register('phone')}
            onChange={(e) => {
              e.target.value = formatPhone(e.target.value);
              register('phone').onChange(e);
            }}
            className={`w-full p-3 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-colors ${
              errors.phone ? 'border-red-500' : 'border-gray-200'
            }`}
            placeholder="(00) 00000-0000"
            maxLength={15}
          />
          {errors.phone && <p className="mt-1 text-sm text-red-500">{errors.phone.message}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Senha</label>
          <input
            {...register('password')}
            type="password"
            className={`w-full p-3 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-colors ${
              errors.password ? 'border-red-500' : 'border-gray-200'
            }`}
            placeholder="******"
          />
          {errors.password && <p className="mt-1 text-sm text-red-500">{errors.password.message}</p>}
        </div>

        <Button
          type="submit"
          disabled={isSubmitting}
          className="w-full mt-6"
        >
          {isSubmitting ? 'Cadastrando...' : 'Cadastrar'}
        </Button>
      </form>
    </div>
  );
};

export default SignUpPage;

export type Role = "SysAdmin" | "TenantOwner" | "TenantUser";

export interface ApiErrorResponse {
  message?: string;
}

export interface AuthResponse {
  accessToken: string;
  refreshToken?: string | null;
  tokenType?: string;
  expireIn?: number;
  email: string;
  uuid: string;
  name: string;
  userId: number;
  role: Role;
  tenantId?: number | null;
  firstLogin: boolean;
  /** Legacy response aliases retained for API compatibility. */
  access_token?: string;
  refresh_token?: string | null;
  tenant_id?: number | null;
}

export interface AuthSession {
  token: string;
  refreshToken: string | null;
  user: AuthResponse | null;
}

export interface Tenant {
  id?: number | null;
  uuid?: string | null;
  businessName?: string | null;
  companyName?: string | null;
  taxId?: string | null;
  email?: string | null;
  phone?: string | null;
  website?: string | null;
  addressLine1?: string | null;
  addressLine2?: string | null;
  locality?: string | null;
  administrativeArea?: string | null;
  postalCode?: string | null;
  countryCode?: string | null;
  province?: string | null;
  city?: string | null;
  zipcode?: string | null;
  paymentGraceDays?: number | null;
  createdAt?: string | null;
  createdBy?: string | null;
  updatedAt?: string | null;
  updatedBy?: string | null;
}

export type TenantInput = Omit<
  Tenant,
  "id" | "uuid" | "createdAt" | "createdBy" | "updatedAt" | "updatedBy"
> & {
  /** Supported by the existing name-only tenant creation flow. */
  name?: string;
};

export interface TenantPlan {
  id?: number | null;
  uuid?: string | null;
  tenantId: number;
  businessPlanId: number;
  paymentDate: string;
  active: boolean;
}

export type TenantPlanInput = Omit<TenantPlan, "id" | "uuid" | "tenantId">;

export interface User {
  id?: number | null;
  uuid?: string | null;
  name?: string | null;
  email: string;
  password?: string | null;
  enabled: boolean;
  firstLogin: boolean;
  role: Role;
  tenantId?: number | null;
  createdAt?: string | null;
  createdBy?: string | null;
  updatedAt?: string | null;
  updatedBy?: string | null;
}

export type UserInput = Pick<User, "email"> &
  Partial<
    Omit<
      User,
      | "id"
      | "uuid"
      | "email"
      | "createdAt"
      | "createdBy"
      | "updatedAt"
      | "updatedBy"
    >
  >;

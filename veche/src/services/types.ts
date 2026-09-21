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
>;

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

export interface Province { id?: number; uuid?: string; ibgeCode?: string | null; acronym: string; name: string; countryCode: string }
export interface City { id?: number; uuid?: string; ibgeCode?: string | null; provinceId: number; name: string }
export interface ImportReport { inserted: number; updated: number; skipped: number; errors: string[] }

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

export interface PagedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}

export interface PageQueryParams {
  page?: number;
  pageSize?: number;
  q?: string;
  sortBy?: string;
  sortDir?: "asc" | "desc";
  [key: string]: unknown;
}

// ==================== Shipping Rate ====================
export interface ShippingRate {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  uf: string;
  priceCents: number;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type ShippingRateInput = Omit<ShippingRate, "id" | "uuid" | "createdAt" | "updatedAt">;

// ==================== Tax Rule ====================
export interface TaxRule {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  name: string;
  ufOrigem?: string | null;
  ufDestino?: string | null;
  ncm?: string | null;
  origemMercadoria?: number | null;
  cst?: string | null;
  aliquotaIcms?: number | null;
  aliquotaIpi?: number | null;
  aliquotaPis?: number | null;
  aliquotaCofins?: number | null;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type TaxRuleInput = Omit<TaxRule, "id" | "uuid" | "createdAt" | "updatedAt">;

// ==================== Customer & Addresses ====================
export interface Customer {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  name: string;
  email: string;
  cpf?: string | null;
  phone?: string | null;
  marketingConsent: boolean;
  active: boolean;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type CustomerInput = Omit<Customer, "id" | "uuid" | "createdAt" | "updatedAt"> & {
  password?: string;
};

export interface CustomerAddress {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  customerId: number;
  recipientName: string;
  phone?: string | null;
  cep: string;
  street: string;
  number: string;
  complement?: string | null;
  neighborhood?: string | null;
  city: string;
  uf: string;
  isDefault: boolean;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type CustomerAddressInput = Omit<CustomerAddress, "id" | "uuid" | "createdAt" | "updatedAt">;

// ==================== Orders, Items & Status Timeline ====================
export interface Orders {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  orderNumber: string;
  customerId: number;
  status: string;
  subtotalCents: number;
  discountCents: number;
  shippingCents: number;
  totalCents: number;
  couponId?: number | null;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type OrdersInput = Omit<Orders, "id" | "uuid" | "createdAt" | "updatedAt">;

export interface OrderItem {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  orderId: number;
  skuId: number;
  skuCode?: string | null;
  productName: string;
  quantity: number;
  unitPriceCents: number;
  discountCents: number;
  ncm?: string | null;
  cfop?: string | null;
  totalCents: number;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type OrderItemInput = Omit<OrderItem, "id" | "uuid" | "createdAt" | "updatedAt">;

export interface OrderStatusHistory {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  orderId: number;
  fromStatus?: string | null;
  toStatus: string;
  actorType: string;
  actorId?: number | null;
  note?: string | null;
  createdAt?: string | null;
}
export type OrderStatusHistoryInput = Omit<OrderStatusHistory, "id" | "uuid" | "createdAt">;

// ==================== Shopping Cart ====================
export interface Cart {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  customerId?: number | null;
  sessionToken?: string | null;
  status: string;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type CartInput = Omit<Cart, "id" | "uuid" | "createdAt" | "updatedAt">;

export interface CartItem {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  cartId: number;
  skuId: number;
  quantity: number;
  unitPriceCents: number;
  totalCents: number;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type CartItemInput = Omit<CartItem, "id" | "uuid" | "createdAt" | "updatedAt">;

// ==================== Marketing: Campaign & Coupon ====================
export interface Campaign {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  name: string;
  description?: string | null;
  campaignType: string;
  startsAt?: string | null;
  endsAt?: string | null;
  budgetLimitCents?: number | null;
  active: boolean;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type CampaignInput = Omit<Campaign, "id" | "uuid" | "createdAt" | "updatedAt">;

export interface CampaignTarget {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  campaignId: number;
  targetType: string;
  targetId: number;
  createdAt?: string | null;
}
export type CampaignTargetInput = Omit<CampaignTarget, "id" | "uuid" | "createdAt">;

export interface Coupon {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  code: string;
  campaignId?: number | null;
  couponType: string;
  value: number;
  minOrderCents?: number | null;
  maxUses?: number | null;
  maxUsesPerCustomer?: number | null;
  startsAt?: string | null;
  expiresAt?: string | null;
  active: boolean;
  createdAt?: string | null;
  updatedAt?: string | null;
}
export type CouponInput = Omit<Coupon, "id" | "uuid" | "createdAt" | "updatedAt">;

export interface CouponRedemption {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  couponId: number;
  orderId: number;
  customerId: number;
  createdAt?: string | null;
}
export type CouponRedemptionInput = Omit<CouponRedemption, "id" | "uuid" | "createdAt">;

// ==================== Catalog & Inventory Extensions ====================
export interface ProductCategory {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  productId: number;
  categoryId: number;
  isPrimary: boolean;
  createdAt?: string | null;
  createdBy?: string | null;
}
export type ProductCategoryInput = Omit<ProductCategory, "id" | "uuid" | "createdAt" | "createdBy">;

export interface SkuAttributeValue {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  productId: number;
  skuId: number;
  productAttributeId: number;
  attributeId: number;
  attributeValueId: number;
  createdAt?: string | null;
  createdBy?: string | null;
  updatedAt?: string | null;
  updatedBy?: string | null;
}
export type SkuAttribute = SkuAttributeValue;
export type SkuAttributeValueInput = Omit<SkuAttributeValue, "id" | "uuid" | "createdAt" | "createdBy" | "updatedAt" | "updatedBy">;
export type SkuAttributeInput = SkuAttributeValueInput;

export interface SkuStock {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  skuId: number;
  quantity: number;
  reserved: number;
  createdAt?: string | null;
  createdBy?: string | null;
  updatedAt?: string | null;
  updatedBy?: string | null;
}
export type SkuStockInput = Omit<SkuStock, "id" | "uuid" | "createdAt" | "createdBy" | "updatedAt" | "updatedBy">;

export interface Person {
  id: number;
  uuid?: string;
  tenantId?: number | null;
  userId: number;
  firstName: string;
  surname?: string | null;
  dateOfBirth?: string | null;
  gender?: string | null;
  avatar?: string | null;
  avatarUrl?: string | null;
  phone?: string | null;
  email?: string | null;
  createdAt?: string | null;
  updatedAt?: string | null;
}

export interface PersonInput {
  tenantId?: number | null;
  userId?: number;
  firstName: string;
  surname?: string | null;
  dateOfBirth?: string | null;
  gender?: string | null;
  avatar?: string | null;
  phone?: string | null;
  email?: string | null;
}

export interface ResourceProfile {
  user: User;
  person: Person | null;
}

export interface AvatarPresignRequest {
  originalFilename: string;
  mimeType: string;
  sizeBytes: number;
}

export interface AvatarPresignResponse {
  uploadUrl: string;
  objectKey: string;
  cdnUrl?: string | null;
}




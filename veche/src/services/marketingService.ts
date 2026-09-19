import { api } from "./api";
import type {
  Campaign,
  CampaignInput,
  CampaignTarget,
  CampaignTargetInput,
  Coupon,
  CouponInput,
  CouponRedemption,
  CouponRedemptionInput,
  PagedResult,
  PageQueryParams,
} from "./types";

export const marketingService = {
  // Campaigns
  async campaignsList() {
    return (await api.get<Campaign[]>("/campaigns")).data;
  },
  async campaignsPaged(params?: PageQueryParams) {
    return (await api.get<PagedResult<Campaign>>("/campaigns/paged", { params })).data;
  },
  async getCampaignById(id: number) {
    return (await api.get<Campaign>(`/campaigns/${id}`)).data;
  },
  async createCampaign(data: CampaignInput) {
    return (await api.post<Campaign>("/campaigns", data)).data;
  },
  async updateCampaign(id: number, data: CampaignInput) {
    return (await api.put<Campaign>(`/campaigns/${id}`, data)).data;
  },

  // Campaign Targets
  async targetsList() {
    return (await api.get<CampaignTarget[]>("/campaign-targets")).data;
  },
  async targetsPaged(params?: PageQueryParams & { campaignId?: number }) {
    return (
      await api.get<PagedResult<CampaignTarget>>("/campaign-targets/paged", { params })
    ).data;
  },
  async createTarget(data: CampaignTargetInput) {
    return (await api.post<CampaignTarget>("/campaign-targets", data)).data;
  },

  // Coupons
  async couponsList() {
    return (await api.get<Coupon[]>("/coupons")).data;
  },
  async couponsPaged(params?: PageQueryParams & { code?: string }) {
    return (await api.get<PagedResult<Coupon>>("/coupons/paged", { params })).data;
  },
  async getCouponById(id: number) {
    return (await api.get<Coupon>(`/coupons/${id}`)).data;
  },
  async createCoupon(data: CouponInput) {
    return (await api.post<Coupon>("/coupons", data)).data;
  },
  async updateCoupon(id: number, data: CouponInput) {
    return (await api.put<Coupon>(`/coupons/${id}`, data)).data;
  },

  // Coupon Redemptions
  async redemptionsList() {
    return (await api.get<CouponRedemption[]>("/coupon-redemptions")).data;
  },
  async redemptionsPaged(params?: PageQueryParams & { couponId?: number; customerId?: number }) {
    return (
      await api.get<PagedResult<CouponRedemption>>("/coupon-redemptions/paged", { params })
    ).data;
  },
  async createRedemption(data: CouponRedemptionInput) {
    return (await api.post<CouponRedemption>("/coupon-redemptions", data)).data;
  },
};

import type { Role } from "@/services/types";

export const ROLES = {
  SYS_ADMIN: "SysAdmin",
  TENANT_OWNER: "TenantOwner",
  TENANT_USER: "TenantUser",
} as const satisfies Record<string, Role>;

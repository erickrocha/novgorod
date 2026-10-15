# NOV-36 / C-040 Implementation Traceability

**Jira:** NOV-36  
**Accepted packet:** `02-system_requirements/changes/NOV-36-remove-mercado-pago/` (local C-040 packet); the separate NOV-37/C-041 packet is archived on Jira attachment 10016.
**Owner Gate 1 acceptance:** 2026-10-15 grooming; Jira comment 10138

The original implementation was directed before Gate 1. These commits were published without Jira/requirement identifiers in their subjects; this record supplies the missing C-040 trace without rewriting history.

| Commit | C-040 behavior in the commit | Accepted trace |
|---|---|---|
| `dd9867e` | Restricts payment settings and migration to PagSeguro; removes Mercado Pago provider, credentials, webhook, mock paths and test double; rejects raw-card-shaped tokens. | NOV-36 / SR-PAY-025 (settings and migration), SR-PAY-026 (provider removal), SR-PAY-030 (raw-card rejection); DES-PAY-12, DES-PAY-15, DES-PAY-16. |
| `ef3c95e` | Uses the PagSeguro SDK encrypted-card flow without a plain-card fallback; supplies the purchase tenant to payment-config and surfaces configuration errors; limits Veche settings to PagSeguro. | NOV-36 / SR-PAY-027 (encrypted-card-only behavior), SR-PAY-028 (tenant-scoped payment config), SR-PAY-029 (Veche settings); DES-PAY-13, DES-PAY-14. |

## Cross-Change Hunks

`ef3c95e` is a mixed implementation commit. Its storefront catalog honesty, cart coupon/error handling, and initial provider installment lookup/UI belong to NOV-37 / C-041, not C-040. Those hunks are independently accepted under SR-STO-001..006 and DES-STO-01..05; the archived NOV-37 AI-G2 report records PASS. Provider-priced interest corrections are in NOV-37 commits `66ce87f` and `4d7422d`. The NOV-36 Gate 2 review excludes these separately owned C-041 behaviors from C-040 scope.

## Gate 1 Follow-Up

The TC-04 component-test additions in `torg/src/components/checkout/__tests__/PagSeguroCardForm.test.tsx` cover `encryptCard` throwing and returning no `encryptedCard` (SR-PAY-027). Focused validation passed 7/7 tests.
/**
 * Strips all non-digit characters from a string.
 */
export function stripNonDigits(value?: string | null): string {
  if (!value) return "";
  return value.replace(/\D/g, "");
}

/**
 * Formats a raw or partially typed string into a masked Brazilian CNPJ (XX.XXX.XXX/XXXX-XX).
 */
export function formatCnpj(value?: string | null): string {
  const digits = stripNonDigits(value).slice(0, 14);
  if (!digits) return "";
  if (digits.length <= 2) return digits;
  if (digits.length <= 5) return `${digits.slice(0, 2)}.${digits.slice(2)}`;
  if (digits.length <= 8) {
    return `${digits.slice(0, 2)}.${digits.slice(2, 5)}.${digits.slice(5)}`;
  }
  if (digits.length <= 12) {
    return `${digits.slice(0, 2)}.${digits.slice(2, 5)}.${digits.slice(5, 8)}/${digits.slice(8)}`;
  }
  return `${digits.slice(0, 2)}.${digits.slice(2, 5)}.${digits.slice(5, 8)}/${digits.slice(8, 12)}-${digits.slice(12, 14)}`;
}

/**
 * Formats a raw or partially typed string into a masked Brazilian CPF (XXX.XXX.XXX-XX).
 */
export function formatCpf(value?: string | null): string {
  const digits = stripNonDigits(value).slice(0, 11);
  if (!digits) return "";
  if (digits.length <= 3) return digits;
  if (digits.length <= 6) return `${digits.slice(0, 3)}.${digits.slice(3)}`;
  if (digits.length <= 9) {
    return `${digits.slice(0, 3)}.${digits.slice(3, 6)}.${digits.slice(6)}`;
  }
  return `${digits.slice(0, 3)}.${digits.slice(3, 6)}.${digits.slice(6, 9)}-${digits.slice(9, 11)}`;
}

/**
 * Validates a Brazilian CNPJ number according to the official check-digit algorithm.
 */
export function isValidCnpj(value?: string | null): boolean {
  const digits = stripNonDigits(value);
  if (digits.length !== 14) return false;

  // Reject sequence of identical digits (e.g. 00000000000000, 11111111111111, etc.)
  if (/^(\d)\1{13}$/.test(digits)) return false;

  const weightsFirst = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
  const weightsSecond = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];

  let sum = 0;
  for (let i = 0; i < 12; i++) {
    sum += Number(digits[i]) * weightsFirst[i];
  }
  let remainder = sum % 11;
  const d1 = remainder < 2 ? 0 : 11 - remainder;
  if (d1 !== Number(digits[12])) return false;

  sum = 0;
  for (let i = 0; i < 13; i++) {
    sum += Number(digits[i]) * weightsSecond[i];
  }
  remainder = sum % 11;
  const d2 = remainder < 2 ? 0 : 11 - remainder;
  if (d2 !== Number(digits[13])) return false;

  return true;
}

/**
 * Formats a raw or partially typed string into a masked Brazilian phone number:
 * - 10 digits: (XX) XXXX-XXXX (landline)
 * - 11 digits: (XX) XXXXX-XXXX (mobile)
 */
export function formatPhone(value?: string | null): string {
  const digits = stripNonDigits(value).slice(0, 11);
  if (!digits) return "";
  if (digits.length <= 2) {
    return `(${digits}`;
  }
  if (digits.length <= 6) {
    return `(${digits.slice(0, 2)}) ${digits.slice(2)}`;
  }
  if (digits.length <= 10) {
    return `(${digits.slice(0, 2)}) ${digits.slice(2, 6)}-${digits.slice(6)}`;
  }
  return `(${digits.slice(0, 2)}) ${digits.slice(2, 7)}-${digits.slice(7, 11)}`;
}

/**
 * Formats a raw or partially typed string into a masked Brazilian postal code (CEP): 00000-000.
 */
export function formatPostalCode(value?: string | null): string {
  const digits = stripNonDigits(value).slice(0, 8);
  if (!digits) return "";
  if (digits.length <= 5) {
    return digits;
  }
  return `${digits.slice(0, 5)}-${digits.slice(5, 8)}`;
}

export const formatCep = formatPostalCode;
export const formatZipcode = formatPostalCode;

import { describe, it, expect } from "vitest";
import i18n from "../index";
import enCommon from "../../locales/en/common.json";
import ptBrCommon from "../../locales/pt-BR/common.json";

function getFlatKeys(obj: Record<string, unknown>, prefix = ''): Record<string, string> {
  const keys: Record<string, string> = {};
  for (const k in obj) {
    const fullKey = prefix ? `${prefix}.${k}` : k;
    const val = obj[k];
    if (typeof val === 'object' && val !== null && !Array.isArray(val)) {
      Object.assign(keys, getFlatKeys(val as Record<string, unknown>, fullKey));
    } else {
      keys[fullKey] = String(val);
    }
  }
  return keys;
}

describe("i18n and locales verification", () => {
  it("has complete parity between en and pt-BR locales with no missing keys", () => {
    const enKeys = getFlatKeys(enCommon);
    const ptKeys = getFlatKeys(ptBrCommon);

    const missingInPt = Object.keys(enKeys).filter((k) => !(k in ptKeys));
    const missingInEn = Object.keys(ptKeys).filter((k) => !(k in enKeys));

    expect(missingInPt).toEqual([]);
    expect(missingInEn).toEqual([]);
  });

  it("correctly translates searchPlaceholder in Portuguese", async () => {
    await i18n.changeLanguage("pt-BR");
    expect(i18n.t("header.searchPlaceholder")).toBe("Pesquisar ou digite um comando...");
    expect(i18n.t("header.toggleSidebar")).toBe("Alternar menu lateral");
  });

  it("correctly translates shipping packaging allowance description in Portuguese", async () => {
    await i18n.changeLanguage("pt-BR");
    expect(i18n.t("operations.shippingSettings.packagingDesc")).toBe(
      "Peso adicional da embalagem e dimensões da caixa a serem adicionados às medidas do item"
    );
    expect(i18n.t("operations.shippingSettings.fixedHelp")).toBe(
      "Utiliza taxas fixas de frete configuradas por estado em Tabelas de Frete"
    );
    expect(i18n.t("operations.shippingSettings.correiosHelp")).toBe(
      "Calcula preço e prazo de entrega em tempo real através da API dos Correios"
    );
  });

  it("correctly translates common session validation and descriptions", async () => {
    await i18n.changeLanguage("pt-BR");
    expect(i18n.t("common.validatingSession")).toBe("Validando sessão...");
    expect(i18n.t("common.description")).toBe("Descrição");

    await i18n.changeLanguage("en");
    expect(i18n.t("common.validatingSession")).toBe("Validating session...");
    expect(i18n.t("common.description")).toBe("Description");
  });
});

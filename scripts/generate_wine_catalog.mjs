import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const XLSX = require("/tmp/wine-catalog-generator/node_modules/xlsx");

const out = path.resolve("data/wines");
fs.mkdirSync(out, { recursive: true });
const wines = [
  ["Tinto", "Merlot", "Serra Gaúcha"], ["Tinto", "Cabernet Sauvignon", "Vale dos Vinhedos"],
  ["Tinto", "Pinot Noir", "Campos de Cima da Serra"], ["Branco", "Chardonnay", "Serra Gaúcha"],
  ["Branco", "Sauvignon Blanc", "Campanha Gaúcha"], ["Rosé", "Moscato", "Vale do São Francisco"],
  ["Espumante", "Chardonnay", "Serra Gaúcha"], ["Espumante", "Pinot Noir", "Vale dos Vinhedos"],
];
const vintages = [2020, 2021, 2022, 2023, 2024];
const categories = ["Tintos", "Brancos", "Rosés", "Espumantes", "Vinhos brasileiros", "Seleção premium"].map((name, i) => ({ external_key: `cat-${i + 1}`, name, slug: name.toLowerCase().replaceAll(" ", "-"), parent_key: "", active: true }));
const attributes = [{ external_key: "attr-color", name: "Cor", display_type: "select" }, { external_key: "attr-grape", name: "Uva", display_type: "select" }, { external_key: "attr-region", name: "Região", display_type: "select" }, { external_key: "attr-vintage", name: "Safra", display_type: "select" }, { external_key: "attr-size", name: "Volume", display_type: "select" }];
const products = [], skus = [], productCategories = [], productAttributes = [], attributeValues = [], skuAttributeValues = [], stock = [];
const valueKeys = new Map();
const addValue = (attributeKey, value) => { const key = `${attributeKey}-${String(value).toLowerCase().replaceAll(" ", "-")}`; if (!valueKeys.has(key)) { valueKeys.set(key, true); attributeValues.push({ external_key: key, attribute_key: attributeKey, value: String(value) }); } return key; };
for (let i = 0; i < 60; i += 1) {
  const [color, grape, region] = wines[i % wines.length]; const vintage = vintages[i % vintages.length]; const key = `wine-${String(i + 1).padStart(3, "0")}`; const slug = `${grape}-${region}-${vintage}-${i + 1}`.toLowerCase().replaceAll(" ", "-");
  products.push({ external_key: key, name: `${grape} Reserva ${vintage} ${region}`, slug, description: `Vinho ${color.toLowerCase()} elaborado com uvas ${grape} da ${region}.`, brand: `Vinícola Aurora ${((i % 6) + 1)}`, color, active: true, ncm: "22042100", cest: "0301200", origem_mercadoria: 0 });
  const category = color === "Tinto" ? "cat-1" : color === "Branco" ? "cat-2" : color === "Rosé" ? "cat-3" : "cat-4"; productCategories.push({ product_key: key, category_key: category, is_primary: true }); productCategories.push({ product_key: key, category_key: "cat-5", is_primary: false });
  const attrs = [["attr-color", color], ["attr-grape", grape], ["attr-region", region], ["attr-vintage", vintage]];
  for (const [attributeKey, value] of attrs) { const valueKey = addValue(attributeKey, value); productAttributes.push({ product_key: key, attribute_key: attributeKey, required: true, sort_order: attrs.findIndex(x => x[0] === attributeKey) }); }
  for (const size of ["750ml", ...(i % 4 === 0 ? ["1500ml"] : [])]) { const skuKey = `${key}-${size.toLowerCase()}`; skus.push({ external_key: skuKey, product_key: key, code: `VIN-${String(i + 1).padStart(3, "0")}-${size.replace("ml", "")}`, variant_key: size, price_cents: 7900 + (i % 10) * 850 + (size === "1500ml" ? 6500 : 0), compare_at_price_cents: null, weight_g: size === "1500ml" ? 2600 : 1350, width_mm: 80, height_mm: size === "1500ml" ? 380 : 315, length_mm: 80, active: true }); for (const [attributeKey, value] of [...attrs, ["attr-size", size]]) skuAttributeValues.push({ sku_key: skuKey, product_key: key, attribute_key: attributeKey, value_key: addValue(attributeKey, value) }); stock.push({ sku_key: skuKey, quantity: 12 + (i % 9) * 4, reserved: i % 3 }); }
}
const sheets = { Categories: categories, Attributes: attributes, AttributeValues: attributeValues, Products: products, ProductCategories: productCategories, ProductAttributes: productAttributes, SKUs: skus, SkuAttributeValues: skuAttributeValues, SkuStock: stock };
const flat = skus.map(sku => { const p = products.find(row => row.external_key === sku.product_key); const c = productCategories.find(row => row.product_key === p.external_key && row.is_primary); return { entity_type: "sku", product_key: p.external_key, sku_key: sku.external_key, category_key: c.category_key, product_name: p.name, slug: p.slug, description: p.description, brand: p.brand, ncm: p.ncm, cest: p.cest, origem_mercadoria: p.origem_mercadoria, code: sku.code, variant_key: sku.variant_key, price_cents: sku.price_cents, compare_at_price_cents: "", weight_g: sku.weight_g, width_mm: sku.width_mm, height_mm: sku.height_mm, length_mm: sku.length_mm, active: true, color: p.color }; });
const headers = Object.keys(flat[0]);
const csvEscape = value => { const text = value == null ? "" : String(value); return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text; };
fs.writeFileSync(path.join(out, "wine_catalog.csv"), `${headers.join(",")}\n${flat.map(row => headers.map(key => csvEscape(row[key])).join(",")).join("\n")}\n`);
const workbook = XLSX.utils.book_new(); for (const [name, rows] of Object.entries(sheets)) XLSX.utils.book_append_sheet(workbook, XLSX.utils.json_to_sheet(rows), name);
XLSX.writeFile(workbook, path.join(out, "wine_catalog.xlsx"));
console.log(`Generated ${products.length} products, ${skus.length} SKUs, ${attributeValues.length} attribute values`);

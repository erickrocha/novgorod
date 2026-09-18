import fs from "node:fs";

const getJson = async (url) => {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url}: ${response.status}`);
  return response.json();
};
const states = await getJson("https://servicodados.ibge.gov.br/api/v1/localidades/estados");
const cities = await getJson("https://servicodados.ibge.gov.br/api/v1/localidades/municipios");
const stateRows = states
  .sort((a, b) => a.id - b.id)
  .map((state) => `${state.id},${state.sigla},${state.nome},BR`);
const cityRows = cities
  .sort((a, b) => a.id - b.id)
  .map((city) => `${city.id},${String(city.id).slice(0, 2)},${city.nome}`);
fs.mkdirSync("data/brazil", { recursive: true });
fs.writeFileSync("data/brazil/provinces.csv", `ibge_code,acronym,name,country_code\n${stateRows.join("\n")}\n`);
fs.writeFileSync("data/brazil/cities.csv", `ibge_code,province_ibge_code,name\n${cityRows.join("\n")}\n`);
console.log(`Generated ${stateRows.length} provinces and ${cityRows.length} cities`);

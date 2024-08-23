import md3 from "./md3-themes.json";

export function colors(theme: string): any {
  const camelToKebabCase = (str: string) =>
    str.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
  return Object.fromEntries(
    Object.entries(md3.schemes[theme]).map(([k, v]) => {
      return [camelToKebabCase(k), v];
    })
  );
}

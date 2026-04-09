// @ts-ignore
import reshaper from "js-arabic-reshaper";

/**
 * Detects if a string contains Arabic characters.
 */
function hasArabic(text: string): boolean {
  const arabicPattern = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/;
  return arabicPattern.test(text);
}

/**
 * Fixes Arabic text for terminal display by reshaping and reversing if necessary.
 */
export function fixArabicDisplay(text: string): string {
  if (!text || !hasArabic(text)) return text;

  try {
    // 1. Reshape the Arabic characters (connect them)
    // js-arabic-reshaper provides the .reshape method
    const reshaped = reshaper.reshape(text);

    // 2. Terminals usually render LTR, so we need to reverse the visual string 
    // to make it look like RTL if it's purely Arabic or mixed.
    // For many terminals, the order needs to be reversed manually.
    return reshaped.split("").reverse().join("");
  } catch (e) {
    console.error("Error reshaping Arabic text:", e);
    return text;
  }
}

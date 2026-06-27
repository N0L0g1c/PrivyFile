import { describe, expect, it } from "vitest";
import { privacyScoreLabel } from "@/components/PrivacyScore";
import { CATEGORY_LABELS, DEFAULT_SETTINGS } from "@/lib/types";

describe("types", () => {
  it("has default settings with preserve_original enabled", () => {
    expect(DEFAULT_SETTINGS.preserve_original).toBe(true);
  });

  it("labels all metadata categories", () => {
    expect(Object.keys(CATEGORY_LABELS)).toHaveLength(5);
    expect(CATEGORY_LABELS.gps).toContain("GPS");
  });
});

describe("privacyScoreLabel", () => {
  it("treats 100 as fully clean", () => {
    expect(privacyScoreLabel(100)).toBe("Fully clean");
  });

  it("treats low scores as exposure risk", () => {
    expect(privacyScoreLabel(20)).toBe("High exposure risk");
  });
});

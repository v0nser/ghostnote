export type PlanId = "pro" | "team";

export type Entitlement =
  | "encrypted_cloud_sync"
  | "advanced_diarization"
  | "custom_ai_prompts"
  | "priority_support"
  | "early_access"
  | "lifetime_discount"
  | "shared_playbooks"
  | "admin_controls"
  | "sso"
  | "team_seats";

export const EARLY_BIRD_DISCOUNT = 0.5;
export const REGULAR_MONTHLY_CENTS = 3000;
export const EARLY_BIRD_MONTHLY_CENTS = 1500;
export const TOTAL_EARLY_BIRD_SPOTS = 500;
export const DEFAULT_CLAIMED = 347;
export const OFFER_STORAGE_KEY = "ghostnote-early-bird-end";
export const OFFER_WINDOW_DAYS = 7;

export const PLAN_FEATURES: Record<
  PlanId,
  { name: string; features: string[]; entitlements: Entitlement[] }
> = {
  pro: {
    name: "Early Bird — PRO",
    features: [
      "Everything in Free tier",
      "Encrypted cloud sync",
      "Advanced speaker diarization",
      "Custom AI prompts",
      "Priority support",
      "Early access to new features",
      "Lifetime 50% discount (locked forever)",
    ],
    entitlements: [
      "encrypted_cloud_sync",
      "advanced_diarization",
      "custom_ai_prompts",
      "priority_support",
      "early_access",
      "lifetime_discount",
    ],
  },
  team: {
    name: "Early Bird — TEAM",
    features: [
      "Everything in Pro",
      "Shared playbooks",
      "Admin controls",
      "SSO",
      "Team seats",
      "Lifetime 50% discount (locked forever)",
    ],
    entitlements: [
      "encrypted_cloud_sync",
      "advanced_diarization",
      "custom_ai_prompts",
      "priority_support",
      "early_access",
      "lifetime_discount",
      "shared_playbooks",
      "admin_controls",
      "sso",
      "team_seats",
    ],
  },
};

export const ENTITLEMENT_COPY: Record<Entitlement, { label: string; detail: string }> = {
  encrypted_cloud_sync: {
    label: "Encrypted cloud sync",
    detail: "Notes and transcripts sync across devices with end-to-end encryption.",
  },
  advanced_diarization: {
    label: "Advanced speaker diarization",
    detail: "Separate speakers in real time with higher accuracy models.",
  },
  custom_ai_prompts: {
    label: "Custom AI prompts",
    detail: "Save interview and meeting playbooks that the local model follows.",
  },
  priority_support: {
    label: "Priority support",
    detail: "Faster replies from the GhostNote team.",
  },
  early_access: {
    label: "Early access",
    detail: "Try new stealth and model features before public release.",
  },
  lifetime_discount: {
    label: "Lifetime 50% discount",
    detail: "Your Early Bird price is locked forever, including upgrades.",
  },
  shared_playbooks: {
    label: "Shared playbooks",
    detail: "Share prompts and answer banks with your interview pod.",
  },
  admin_controls: {
    label: "Admin controls",
    detail: "Manage seats, devices, and retention from one dashboard.",
  },
  sso: {
    label: "SSO",
    detail: "Sign in with your company identity provider.",
  },
  team_seats: {
    label: "Team seats",
    detail: "Invite teammates at the locked Early Bird rate.",
  },
};

export function priceForPlan(plan: PlanId, expired: boolean) {
  const monthly = expired ? REGULAR_MONTHLY_CENTS : EARLY_BIRD_MONTHLY_CENTS;
  return {
    monthlyCents: monthly,
    regularCents: REGULAR_MONTHLY_CENTS,
    perUser: plan === "team",
    discountPct: expired ? 0 : 50,
  };
}

export function makeDiscountCode() {
  const part = Math.random().toString(36).slice(2, 8).toUpperCase();
  return `EARLY50-${part}`;
}

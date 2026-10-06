import { isPolarConfigured } from "@/lib/polar";
import { isStripeConfigured } from "@/lib/stripe";

export type PaymentProvider = "polar" | "stripe" | "demo" | "none";

export function isProductionRuntime() {
  return process.env.VERCEL === "1" || process.env.NODE_ENV === "production";
}

export function getPaymentProvider(): PaymentProvider {
  if (isPolarConfigured()) return "polar";
  if (isStripeConfigured()) return "stripe";
  return isProductionRuntime() ? "none" : "demo";
}

export function paymentCopy(provider: PaymentProvider) {
  switch (provider) {
    case "polar":
      return "Pay with card worldwide. Polar is the merchant of record and deposits payouts to your bank.";
    case "stripe":
      return "Pay securely with Stripe Checkout.";
    case "demo":
      return "Local demo mode: no card is charged. Set Polar keys to take real payments.";
    default:
      return "Payments are not configured yet. Add Polar keys on the server.";
  }
}

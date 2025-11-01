import { z } from "zod"

//SignupSchema
export const SignupSchema = z.object({
  email: z
    .string()
    .email("Invalid email address"),
})

//Signin schema
export const SigninSchema = z.object({
  email: z
    .string()
    .email("Invalid email address"),
  password: z
    .string()
    .min(8, "Password must be at least 8 characters long")
    .max(20, "Password must be at most 20 characters long")
    .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,20}$/, "Password must include at least one uppercase letter, one lowercase letter, one digit, and one special character"),
})

//verifyotp schema 
export const VerifyOtpSchema = z.object({
  email: z
    .string()
    .email("Invalid email address"),
  otp: z
    .string()
    .length(6, "OTP must be 6 digits"),
  password: z
    .string()
    .min(8, "Password must be atleast 8 characters long!")
    .max(20, "Password must be at most 20 characters long")
    .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,20}$/,
      "Password must include at least one uppercase letter, one lowercase letter, one digit, and one special character"
    ),
  name: z
    .string()
    .min(2, "Name must be at least 3 characters long")
    .max(50, "Name must be at most 50 characters long"),
});

export const PriceUpdateSchema = z.object({
  asset: z.string(),
  price: z.number(),
  timestamp: z.number(),
})

export const createOrderSchema = z.object({
  asset: z.string().min(1, "Asset is required"),
  side: z.enum(["buy", "sell"]),
  margin: z.number().positive("Margin must be a positive number"),
  leverage: z.number().int().positive("Leverage must be a positive integer"),
  slippage: z.number().int().nonnegative("Slippage must be a non-negative integer"),
  orderType: z.optional(z.enum(["market", "limit"])),
  // accept omitted key or explicit null from clients; superRefine enforces presence for limit orders
  limitPrice: z.number().positive("limitPrice must be a positive number").nullable().optional(),
  stopLossPercent: z.optional(z.number()),
  takeProfitPercent: z.optional(z.number()),
  tradeTerm: z.optional(z.enum(["INTRAHOUR", "INTRADAY", "WEEK", "MONTH", "YEAR"])),
  quantity: z.optional(z.number().positive("Quantity must be a positive number")),
  timeInForce: z.optional(z.enum(["IOC", "FOK", "DAY", "GTC", "EXPIRE_AT"])),
  expiryTimestamp: z.optional(z.number().int().nonnegative("expiryTimestamp must be a non-negative integer")),
}).superRefine((data, ctx) => {
  // require limitPrice for limit orders
  if (data.orderType === "limit" && (data.limitPrice == null)) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      message: "limitPrice is required for limit orders",
      path: ["limitPrice"],
    });
  }

  // validate percent bounds if provided
  if (data.stopLossPercent != null) {
    if (typeof data.stopLossPercent !== "number" || data.stopLossPercent <= 0 || data.stopLossPercent >= 100) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: "stopLossPercent must be > 0 and < 100",
        path: ["stopLossPercent"],
      });
    }
  }

  if (data.takeProfitPercent != null) {
    if (typeof data.takeProfitPercent !== "number" || data.takeProfitPercent <= 0 || data.takeProfitPercent >= 100) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: "takeProfitPercent must be > 0 and < 100",
        path: ["takeProfitPercent"],
      });
    }
  }

  // expiryTimestamp sanity: if provided ensure it's in the future
  if (data.expiryTimestamp != null) {
    const now = Date.now();
    if (data.expiryTimestamp <= now) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: "expiryTimestamp must be in the future",
        path: ["expiryTimestamp"],
      });
    }
  }
});

export const closeOrderSchema = z.object({
  orderId: z.string().uuid("Invalid order ID"),  // Assuming UUID format
});


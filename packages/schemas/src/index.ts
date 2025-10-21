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
    type: z.enum(["long", "short"]),  // Removed invalid required_error
    margin: z.number().int().positive("Margin must be a positive integer"),
    leverage: z.number().int().positive("Leverage must be a positive integer"),
    slippage: z.number().int().positive("Slippage must be a positive integer"),
});

export const closeOrderSchema = z.object({
    orderId: z.string().uuid("Invalid order ID"),  // Assuming UUID format
});
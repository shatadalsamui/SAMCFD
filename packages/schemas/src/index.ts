import { z } from "zod"

//SignupSchema
export const SignupSchema = z.object({
    email: z
        .string()
        .email("Invalid email address"),
    password: z
        .string()
        .min(8, "Password must be atleast 8 characters long!")
        .max(20, "Password must be at most 20 characters long")
        .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,20}$/,
            "Password must include at least one uppercase letter, one lowercase letter, one digit, and one special character"
        ),
    username: z
        .string()
        .min(10, "Username must be at least 10 characters long")
        .max(20, "Username must be at most 20 characters long")
})
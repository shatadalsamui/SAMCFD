// Authentication Router
// Organizes authentication-related routes such as signup, signin, and OTP verification.

import { Router } from "express";
import signupRouter from "./signup";
import signinRouter from "./signin";
import verifyOtpRouter from "./verifyOtp";
import logoutRouter from "./logout"; // Add this
import { requireAuth } from "../../../middleware/auth"; // Add this

const authRouter = Router();

authRouter.use("/signup", signupRouter); // /api/v1/auth/signup
authRouter.use("/signin", signinRouter); // /api/v1/auth/signin
authRouter.use("/verify", verifyOtpRouter); // /api/v1/auth/verify
authRouter.use("/logout", logoutRouter); // /api/v1/auth/logout
authRouter.get("/me", requireAuth, (req, res) => res.json({ user: (req as any).user })); // Add this: Protected /api/v1/auth/me

export default authRouter;
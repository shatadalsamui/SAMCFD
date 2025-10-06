// Authentication Router
// Organizes authentication-related routes such as signup, signin, and OTP verification.

import { Router } from "express";
import signupRouter from "./signup";
import signinRouter from "./signin";
import verifyOtpRouter from "./verifyOtp";

const authRouter = Router();

authRouter.use("/signup", signupRouter); // /api/v1/auth/signup
authRouter.use("/signin", signinRouter); // /api/v1/auth/signin
authRouter.use("/verify", verifyOtpRouter); // /api/v1/auth/verify

export default authRouter;
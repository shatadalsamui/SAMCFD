// Verify OTP Router
// Handles OTP verification requests for user authentication.

import { Router } from "express";
import { verifyOtp } from "../../../controllers/signup/verifyOtp";

const verifyOtpRouter = Router();

verifyOtpRouter.post("/", verifyOtp); // POST /api/v1/auth/verify

export default verifyOtpRouter;
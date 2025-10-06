import { Router } from "express";
import { signUp } from "../../../controllers/signup/signup";

const signupRouter = Router();

signupRouter.post("/", signUp); // POST /api/v1/auth/signup

export default signupRouter;
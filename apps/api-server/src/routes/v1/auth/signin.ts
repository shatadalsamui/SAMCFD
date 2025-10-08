// Signin Router
// Placeholder for signin functionality, to be implemented.

import { Router } from "express";
import { signIn } from "../../../controllers/signin/signin";

const signinRouter = Router();

signinRouter.post("/", signIn);

export default signinRouter;
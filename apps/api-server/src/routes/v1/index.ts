import { Router } from "express";
import authRouter from "./auth/index"; // Import the auth sub-router

const v1Router = Router();

// Mount auth routes under /auth
v1Router.use("/auth", authRouter);

export default v1Router;
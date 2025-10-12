// API Version 1 Router
// Defines and organizes routes for version 1 of the API.

import { Router } from "express";
import authRouter from "./auth/index"; // Import the auth sub-router
import pricesRouter from "./prices/index"; // Import the prices sub-router

const v1Router = Router();

// Mount auth routes under /auth
v1Router.use("/auth", authRouter);

// Mount prices routes under /prices
v1Router.use("/prices", pricesRouter);

export default v1Router;
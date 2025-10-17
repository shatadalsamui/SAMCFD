import { Router } from "express";
import { getBalanceController } from "../../../controllers/balance/balance";
import { requireAuth } from "../../../middleware/auth";

const balanceRouter = Router();

balanceRouter.get("/", requireAuth, getBalanceController);

export default balanceRouter;
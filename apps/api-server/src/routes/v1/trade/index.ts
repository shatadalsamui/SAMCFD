import { Router } from "express";
import { createOrderController } from "../../../controllers/trade/createOrder";
import { closeOrderController } from "../../../controllers/trade/closeOrder";
import { requireAuth } from "../../../middleware/auth";

const tradeRouter = Router();

tradeRouter.post("/create", requireAuth, createOrderController);
tradeRouter.post("/close", requireAuth, closeOrderController);

export default tradeRouter;
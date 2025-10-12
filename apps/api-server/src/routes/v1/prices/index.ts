import { Router } from "express";
import { getLatestPrices } from "../../../controllers/prices/prices";

const pricesRouter = Router();

pricesRouter.get("/", getLatestPrices); // GET /api/v1/prices

export default pricesRouter;
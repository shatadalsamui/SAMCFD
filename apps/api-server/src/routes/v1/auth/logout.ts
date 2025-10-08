import { Router } from "express";
import { logout } from "../../../controllers/logout/logout";

const logoutRouter = Router();
logoutRouter.post("/", logout);

export default logoutRouter;
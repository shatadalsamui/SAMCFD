import Router from "express";

import { SignupSchema } from "@repo/schemas"

const v1Router = Router();

v1Router.post("/signup", (req, res) => {

    const { data, success, error } = SignupSchema.safeParse(req.body);

    if (!success) {
        res.status(411).json({
            message: "Validation failed",
            errors: error.issues
        })
    }
    
})

v1Router.post("/signin", (req, res) => {

})

v1Router.post("/signin/post", (req, res) => {

})

export default v1Router;
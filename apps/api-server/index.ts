import express from "express";
import v1Router from "./routes/v1"
import cors from "express"

const app = express();

app.use(cors());
app.use(express.json());
app.use("/api/v1", v1Router);


app.listen(3000);


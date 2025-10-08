// OTP Mail Service
// Sends OTP emails to users using Nodemailer with Gmail.
import nodemailer from "nodemailer";

const transporter = nodemailer.createTransport({
    service: "gmail", // Use Gmail service
    auth: {
        user: process.env.GMAIL_USER!, // Gmail address
        pass: process.env.GMAIL_APP_PASSWORD!, // Gmail App Password
    },
});

export const sendOtpMail = async (email: string, otp: string) => {
    try {
        const info = await transporter.sendMail({
            from: process.env.GMAIL_USER!, // Sender Gmail address
            to: email, // Receiver email
            subject: "Your OTP Code - SAMCFD", // Subject line
            html: `<p>Your OTP for SAMCFD Account creation is: <strong>${otp}</strong></p>`, // HTML body
        });
        console.log(`OTP email sent to ${email}: ${info.messageId}`);
    } catch (error) {
        console.error(`Error sending OTP email to ${email}:`, error);
        throw new Error("Failed to send OTP email");
    }
};
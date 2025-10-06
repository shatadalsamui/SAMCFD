import { Resend } from "resend";

const resend = new Resend(process.env.RESEND_API_KEY);

export const sendOtpMail = async (email: string, otp: string) => {
    try {
        await resend.emails.send({
            from: process.env.RESEND_FROM_EMAIL!, // Mandatory sender email
            to: [email], // Array format as shown in docs
            subject: "Your OTP Code - SAMCFD",
            html: `<p>Your OTP for SAMCFD Account creation is: <strong>${otp}</strong></p>`,
        });
        console.log(`OTP email sent to ${email}`);
    } catch (error) {
        console.error(`Error sending OTP email to ${email}:`, error);
        throw new Error("Failed to send OTP email");
    }
};
# Signup Flow Documentation

## Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant API Server
    participant Kafka
    participant DB Processor
    participant Database
    participant Redis
    participant Email Service

    User->>API Server: POST /api/v1/auth/signup (email)
    API Server->>API Server: Validate input (Zod schema)
    API Server->>Kafka: user-existence-check (email, correlationId)
    Kafka->>DB Processor: Consume message
    DB Processor->>Database: Check if user exists by email
    Database-->>DB Processor: Return exists: true/false
    DB Processor->>Kafka: user-existence-response (exists, correlationId)
    Kafka-->>API Server: Consume response
    alt User exists
        API Server-->>User: 409 User already exists
    else User does not exist
        API Server->>API Server: Generate 6-digit OTP
        API Server->>API Server: bcrypt.hash(OTP, 12)
        API Server->>Redis: SET otp:{email} hashedOTP EX 300
        API Server->>Email Service: Send OTP email via Nodemailer
        Email Service-->>API Server: Email sent (awaited)
        API Server-->>User: 200 {message: "OTP sent to your email"}
    end
```

## Flow Description

1. **Input Validation**: Validates email using Zod schema.
2. **User Existence Check**: Kafka request to DB processor to check if user already exists.
3. **Database Query**: DB processor queries database for existing user.
4. **OTP Generation**: If user doesn't exist, generates a 6-digit OTP.
5. **OTP Storage**: Hashes and stores OTP in Redis with 5-minute expiration.
6. **Email Sending**: Sends OTP via Gmail SMTP (Nodemailer), blocking until sent.
7. **Response**: Confirms OTP sent to email.

## Performance Notes

- Total time: ~3.5s
- Main bottleneck: Email sending (~3-4s via SMTP)
- Fast components: Validation, OTP gen, Redis store (<50ms)

## Error Handling

- Invalid input: 411 Validation failed
- User exists: 409 User already exists
- Kafka/DB errors: 500 Failed to verify user existence
- Email send failure: 500 Internal server error

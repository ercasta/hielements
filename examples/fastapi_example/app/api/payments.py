"""
Payment API Example - FastAPI Microservice with Authentication

This example demonstrates a FastAPI microservice with proper authentication
on all POST and GET endpoints, following security best practices.
"""

from fastapi import FastAPI, Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer
from pydantic import BaseModel
from typing import Optional, List
import uvicorn

from .auth import get_current_user, User
from .models import PaymentRequest, PaymentResponse, Payment

app = FastAPI(title="Payment API", version="1.0.0")

# OAuth2 scheme for token-based authentication
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")


@app.get("/")
async def root():
    """Public endpoint - health check."""
    return {"status": "ok", "service": "payment-api"}


@app.get("/api/payments", response_model=List[PaymentResponse])
async def list_payments(
    current_user: User = Depends(get_current_user),
    skip: int = 0,
    limit: int = 10
):
    """
    List all payments for the authenticated user.
    
    Authentication: Required (via Depends)
    """
    # In real implementation, fetch from database
    return [
        PaymentResponse(
            id="pay_123",
            amount=100.00,
            currency="USD",
            status="completed",
            user_id=current_user.id
        )
    ]


@app.get("/api/payments/{payment_id}", response_model=PaymentResponse)
async def get_payment(
    payment_id: str,
    current_user: User = Depends(get_current_user)
):
    """
    Get a specific payment by ID.
    
    Authentication: Required (via Depends)
    """
    # In real implementation, fetch from database
    if payment_id != "pay_123":
        raise HTTPException(status_code=404, detail="Payment not found")
    
    return PaymentResponse(
        id=payment_id,
        amount=100.00,
        currency="USD",
        status="completed",
        user_id=current_user.id
    )


@app.post("/api/payments", response_model=PaymentResponse, status_code=status.HTTP_201_CREATED)
async def create_payment(
    payment: PaymentRequest,
    current_user: User = Depends(get_current_user)
):
    """
    Create a new payment.
    
    Authentication: Required (via Depends)
    """
    # In real implementation, process payment and store in database
    return PaymentResponse(
        id="pay_new_123",
        amount=payment.amount,
        currency=payment.currency,
        status="pending",
        user_id=current_user.id
    )


@app.post("/api/payments/{payment_id}/cancel", response_model=PaymentResponse)
async def cancel_payment(
    payment_id: str,
    current_user: User = Depends(get_current_user)
):
    """
    Cancel a payment.
    
    Authentication: Required (via Depends)
    """
    # In real implementation, cancel payment in database
    return PaymentResponse(
        id=payment_id,
        amount=100.00,
        currency="USD",
        status="cancelled",
        user_id=current_user.id
    )


@app.get("/health/liveness")
async def liveness_check():
    """Liveness probe - no auth required."""
    return {"status": "alive"}


@app.get("/health/readiness")
async def readiness_check():
    """Readiness probe - no auth required."""
    # In real implementation, check database connectivity, etc.
    return {"status": "ready", "database": "ok"}


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8080)

"""
Data models for the Payment API.

Defines Pydantic models for request/response schemas.
"""

from pydantic import BaseModel, Field
from typing import Optional, Literal
from datetime import datetime


class PaymentRequest(BaseModel):
    """Request model for creating a payment."""
    amount: float = Field(..., gt=0, description="Payment amount (must be positive)")
    currency: str = Field(..., min_length=3, max_length=3, description="Currency code (ISO 4217)")
    description: Optional[str] = Field(None, max_length=500, description="Payment description")
    metadata: Optional[dict] = Field(None, description="Additional metadata")
    
    class Config:
        schema_extra = {
            "example": {
                "amount": 100.00,
                "currency": "USD",
                "description": "Payment for order #12345",
                "metadata": {"order_id": "12345"}
            }
        }


class PaymentResponse(BaseModel):
    """Response model for payment operations."""
    id: str = Field(..., description="Unique payment identifier")
    amount: float = Field(..., description="Payment amount")
    currency: str = Field(..., description="Currency code")
    status: Literal["pending", "completed", "failed", "cancelled"] = Field(..., description="Payment status")
    user_id: str = Field(..., description="User ID who created the payment")
    created_at: Optional[datetime] = Field(None, description="Creation timestamp")
    updated_at: Optional[datetime] = Field(None, description="Last update timestamp")
    
    class Config:
        schema_extra = {
            "example": {
                "id": "pay_1234567890",
                "amount": 100.00,
                "currency": "USD",
                "status": "completed",
                "user_id": "user_123",
                "created_at": "2024-01-01T12:00:00Z",
                "updated_at": "2024-01-01T12:01:00Z"
            }
        }


class Payment(BaseModel):
    """Internal payment model with additional fields."""
    id: str
    amount: float
    currency: str
    status: str
    user_id: str
    description: Optional[str] = None
    metadata: Optional[dict] = None
    created_at: datetime
    updated_at: datetime
    
    class Config:
        orm_mode = True

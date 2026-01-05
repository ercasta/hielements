"""
Authentication module for the Payment API.

Provides user authentication and authorization functionality.
"""

from fastapi import Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm
from pydantic import BaseModel
from typing import Optional
import jwt
from datetime import datetime, timedelta

# OAuth2 scheme
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/auth/token")

# Secret key for JWT (in production, use environment variable)
SECRET_KEY = "your-secret-key-here"
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30


class User(BaseModel):
    """User model."""
    id: str
    username: str
    email: str
    is_active: bool = True


class TokenData(BaseModel):
    """Token data model."""
    username: Optional[str] = None


def verify_token(token: str) -> TokenData:
    """Verify and decode JWT token."""
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        username: str = payload.get("sub")
        if username is None:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Invalid authentication credentials",
                headers={"WWW-Authenticate": "Bearer"},
            )
        return TokenData(username=username)
    except jwt.PyJWTError:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid authentication credentials",
            headers={"WWW-Authenticate": "Bearer"},
        )


def get_current_user(token: str = Depends(oauth2_scheme)) -> User:
    """
    Get the current authenticated user from the token.
    
    This is the main authentication dependency used throughout the API.
    """
    token_data = verify_token(token)
    
    # In real implementation, fetch user from database
    # For now, return a mock user
    user = User(
        id="user_123",
        username=token_data.username or "testuser",
        email="test@example.com",
        is_active=True
    )
    
    if not user.is_active:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Inactive user"
        )
    
    return user


def create_access_token(data: dict, expires_delta: Optional[timedelta] = None) -> str:
    """Create a new JWT access token."""
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=15)
    
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt


async def authenticate_user(username: str, password: str) -> Optional[User]:
    """
    Authenticate a user with username and password.
    
    In production, verify against database with hashed passwords.
    """
    # Mock authentication - in production, check database
    if username == "testuser" and password == "testpass":
        return User(
            id="user_123",
            username=username,
            email="test@example.com",
            is_active=True
        )
    return None

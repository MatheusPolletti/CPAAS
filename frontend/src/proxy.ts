import { withAuth } from "next-auth/middleware";
import { NextResponse } from "next/server";

export default withAuth(
  function middleware(req) {
    const { pathname } = req.nextUrl;
    const isAuth = !!req.nextauth.token;

    const isPublicRoute = pathname === "/login" || pathname === "/register";

    if (pathname === "/") {
      if (!isAuth) {
        return NextResponse.redirect(new URL("/login", req.url));
      }
      return NextResponse.redirect(new URL("/sms", req.url));
    }

    if (isPublicRoute && isAuth) {
      return NextResponse.redirect(new URL("/sms", req.url));
    }

    if (!isAuth && !isPublicRoute) {
      return NextResponse.redirect(new URL("/login", req.url));
    }

    return NextResponse.next();
  },
  {
    callbacks: {
      authorized: () => true,
    },
  }
);

export const config = {
  matcher: [
    "/((?!api/auth|_next/static|_next/image|favicon.ico).*)",
  ],
};
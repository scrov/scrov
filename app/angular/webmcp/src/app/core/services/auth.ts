import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, tap, catchError, firstValueFrom, of } from 'rxjs';
import { User } from '../models/user.model';
import { declareWebMcpTool } from '@mcp-b/webmcp'; 

@Injectable({ providedIn: 'root' })
export class Auth {
  private readonly http = inject(HttpClient);

  public readonly user = signal<User | null>(null);
  public readonly isAuthenticated = computed<boolean>(() => this.user() !== null);
  public readonly isCustomer = computed<boolean>(() => this.user()?.role === 'CUSTOMER');
  public readonly isAdmin = computed<boolean>(() => this.user()?.role === 'PIZZERIA_ADMIN');

  public constructor() {
    declareWebMcpTool({
      name: 'get_current_user',
      description:
        'Returns whether someone is currently signed in, and if so, their name, email, and role.',
      inputSchema: { type: 'object', properties: {} },
      execute: () => {
        const currentUser = this.user();
        if (!currentUser) {
          return { content: [{ type: 'text', text: 'No user is currently signed in.' }] };
        }
        return {
          content: [
            {
              type: 'text',
              text: `Signed in as ${currentUser.name} (${currentUser.email}), role: ${currentUser.role}.`,
            },
          ],
        };
      },
    });

    // Deliberately not reusing the profile page's logout flow, which does a hard
    // `window.location.href` reload: that would tear down the page (and this tool's own
    // WebMCP registration) mid-call. Sign out in place instead and update the signal directly,
    // since `logout()` itself doesn't touch `user()`.
    declareWebMcpTool({
      name: 'logout',
      description: 'Signs the current user out.',
      inputSchema: { type: 'object', properties: {} },
      execute: async () => {
        if (!this.isAuthenticated()) {
          return { content: [{ type: 'text', text: 'No user is currently signed in.' }] };
        }
        await firstValueFrom(this.logout());
        this.user.set(null);
        return { content: [{ type: 'text', text: 'Signed out.' }] };
      },
    });
  }

  public init(): Observable<User | null> {
    return this.http.get<User>('/api/auth/me').pipe(
      tap((user) => {
        this.user.set(user);
      }),
      catchError(() => {
        this.user.set(null);
        return of(null);
      }),
    );
  }

  public register(email: string, password: string): Observable<User> {
    return this.http
      .post<User>('/api/auth/register', { email, password })
      .pipe(tap((user) => this.user.set(user)));
  }

  public registerPizzeriaOwner(email: string, password: string): Observable<User> {
    return this.http
      .post<User>('/api/auth/register-pizzeria-owner', { email, password })
      .pipe(tap((user) => this.user.set(user)));
  }

  public login(email: string, password: string): Observable<User> {
    return this.http
      .post<User>('/api/auth/login', { email, password })
      .pipe(tap((user) => this.user.set(user)));
  }

  public logout(): Observable<void> {
    return this.http.post<void>('/api/auth/logout', {});
  }
}

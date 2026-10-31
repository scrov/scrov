import { Routes } from '@angular/router';
import { roleGuard } from '../../core/guards/role/role.guard';
import { ROLES } from '../auth/role.model';
import { orderDetailWebMcpTools, orderListWebMcpTools } from './order.webmcp-tools';

export const ordersRoutes: Routes = [
  {
    path: 'admin',
    title: 'Orders - Admin',
    canMatch: [roleGuard(ROLES.PIZZERIA_ADMIN)],
    loadComponent: () =>
      import('./pages/admin-order-list-page/admin-order-list-page').then(
        (m) => m.AdminOrderListPage,
      ),
  },
  {
    path: '',
    providers: [...orderListWebMcpTools],
    loadComponent: () =>
      import('./pages/order-list-page/order-list-page').then((m) => m.OrdersListPage),
  },
  {
    path: ':id',
    providers: [...orderDetailWebMcpTools],
    loadComponent: () =>
      import('./pages/order-details-page/order-details-page').then((m) => m.OrderDetailPage),
  },
];

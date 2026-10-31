import { Routes } from '@angular/router';
import { checkoutDeactivateGuard } from '../../core/guards/checkout-deactivate/checkout-deactivate.guard';
import { CheckoutPage } from './pages/checkout-page/checkout-page';
import { checkoutWebMcpTools } from './checkout.webmcp-tools';

export const checkoutRoutes: Routes = [
  {
    path: '',
    canDeactivate: [checkoutDeactivateGuard],
    providers: [...checkoutWebMcpTools],
    component: CheckoutPage,
  },
];

import {Suspense} from "react";
import type React from "react";
import {Navigate, Route, Routes} from "react-router-dom";
import AppLayout from "@/layout/AppLayout.tsx";
import UserProfiles from "@/pages/UserProfiles.tsx";
import Calendar from "@/pages/Calendar";
import BarChart from "@/pages/Charts/BarChart";
import LineChart from "@/pages/Charts/LineChart";
import Home from "@/pages/Dashboard/Ecommerce";
import FormElements from "@/pages/Forms/FormElements";
import Blank from "@/pages/OtherPage/Blank";
import BasicTables from "@/pages/Tables/BasicTables";
import Alerts from "@/pages/UiElements/Alerts";
import Avatars from "@/pages/UiElements/Avatars";
import Badges from "@/pages/UiElements/Badges";
import Buttons from "@/pages/UiElements/Buttons";
import Images from "@/pages/UiElements/Images";
import Videos from "@/pages/UiElements/Videos";
import {ScrollToTop} from "@/components/common/ScrollToTop.tsx";
import Users from "@/pages/Management/Users";
import UserForm from "@/pages/Management/UserForm";
import Tenants from "@/pages/Management/Tenants";
import TenantForm from "@/pages/Management/TenantForm";
import CatalogList from "@/pages/Catalog/CatalogList";
import CatalogForm from "@/pages/Catalog/CatalogForm";
import ProductImagesPage from "@/pages/Catalog/ProductImagesPage";
import Locations from "@/pages/Management/Locations";
import Orders from "@/pages/Sales/Orders";
import Carts from "@/pages/Sales/Carts";
import Customers from "@/pages/Customers/Customers";
import CustomerForm from "@/pages/Customers/CustomerForm";
import ProfileEdit from "@/pages/Profile/ProfileEdit";
import PersonAddressForm from "@/pages/Profile/PersonAddressForm";
import Campaigns from "@/pages/Marketing/Campaigns";
import Coupons from "@/pages/Marketing/Coupons";
import ShippingRates from "@/pages/Operations/ShippingRates";
import TaxRules from "@/pages/Operations/TaxRules";
import Inventory from "@/pages/Operations/Inventory";
import ProductCategories from "@/pages/Catalog/ProductCategories";
import SkuAttributes from "@/pages/Catalog/SkuAttributes";
import { useAppSelector } from "@/store/hooks";
import { ROLES } from "@/utils/enums";

const SysAdminOnly = ({ children }: { children: React.ReactNode }) => {
    const role = useAppSelector((state) => state.auth.user?.role);
    return role === ROLES.SYS_ADMIN ? <>{children}</> : <Navigate to="/" replace />;
};


export const ProtectedRoutes = () => (
    <Suspense fallback={<div className="route-loading-state">Loading...</div>}>
        <ScrollToTop />
        <Routes>
            <Route element={<AppLayout />}>
                <Route index path="/" element={<Home />} />

                {/* Profile */}
                <Route path="/profile" element={<UserProfiles />} />
                <Route path="/profile/edit" element={<ProfileEdit />} />
                <Route path="/profile/addresses/new" element={<PersonAddressForm />} />
                <Route path="/profile/addresses/:id/edit" element={<PersonAddressForm />} />

                {/* Users & Tenants */}
                <Route path="/users" element={<Users />} />
                <Route path="/users/new" element={<UserForm />} />
                <Route path="/users/:id/edit" element={<UserForm />} />
                <Route path="/tenants" element={<Tenants />} />
                <Route path="/tenants/new" element={<TenantForm />} />
                <Route path="/tenants/:id/edit" element={<TenantForm />} />
                <Route path="/catalog/product-categories" element={<ProductCategories />} />
                <Route path="/catalog/sku-attributes" element={<SkuAttributes />} />
                <Route path="/catalog/products/:id/photos" element={<ProductImagesPage />} />
                <Route path="/catalog/products/:id/images" element={<ProductImagesPage />} />
                <Route path="/catalog/:kind" element={<CatalogList />} />
                <Route path="/catalog/:kind/new" element={<CatalogForm />} />
                <Route path="/catalog/:kind/:id/edit" element={<CatalogForm />} />

                {/* Sales & Orders */}
                <Route path="/orders" element={<Orders />} />
                <Route path="/carts" element={<Carts />} />

                {/* Customers */}
                <Route path="/customers" element={<Customers />} />
                <Route path="/customers/new" element={<CustomerForm />} />
                <Route path="/customers/:id/edit" element={<CustomerForm />} />

                {/* Marketing */}
                <Route path="/marketing/campaigns" element={<Campaigns />} />
                <Route path="/marketing/coupons" element={<Coupons />} />

                {/* Operations & Settings */}
                <Route path="/operations/inventory" element={<Inventory />} />
                <Route path="/operations/shipping-rates" element={<ShippingRates />} />
                <Route path="/operations/tax-rules" element={<TaxRules />} />

                <Route path="/system-settings/provinces" element={<SysAdminOnly><Locations kind="provinces" /></SysAdminOnly>} />
                <Route path="/system-settings/cities" element={<SysAdminOnly><Locations kind="cities" /></SysAdminOnly>} />
                <Route path="/calendar" element={<Calendar />} />
                <Route path="/blank" element={<Blank />} />

                {/* Forms */}
                <Route path="/form-elements" element={<FormElements />} />

                {/* Tables */}
                <Route path="/basic-tables" element={<BasicTables />} />

                {/* Ui Elements */}
                <Route path="/alerts" element={<Alerts />} />
                <Route path="/avatars" element={<Avatars />}     />
                <Route path="/badge" element={<Badges />} />
                <Route path="/buttons" element={<Buttons />} />
                <Route path="/images" element={<Images />} />
                <Route path="/videos" element={<Videos />} />

                {/* Charts */}
                <Route path="/line-chart" element={<LineChart />} />
                <Route path="/bar-chart" element={<BarChart />} />
            </Route>
            <Route path="/login" element={<Navigate to="/" replace />} />
            <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
    </Suspense>
);

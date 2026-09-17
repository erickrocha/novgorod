import {Suspense} from "react";
import {Route, Routes} from "react-router";
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


export const ProtectedRoutes = () => (
    <Suspense fallback={<div className="route-loading-state">Loading...</div>}>
        <Routes>
            <Route element={<AppLayout />}>
                <Route index path="/" element={<Home />} />

                {/* Others Page */}
                <Route path="/profile" element={<UserProfiles />} />
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
        </Routes>
    </Suspense>

)
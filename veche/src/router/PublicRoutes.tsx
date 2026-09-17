import { Routes, Route, Navigate } from 'react-router-dom';
import SignIn from "@/pages/AuthPages/SignIn.tsx";


export const PublicRoutes =()=> (
    <Routes>
        <Route path="/login" element={<SignIn />} />
        <Route path="*" element={<Navigate to="/login" replace />} />
    </Routes>
)
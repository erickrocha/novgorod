import { X } from "lucide-react";
import ProductImagesManager from "./ProductImagesManager";

interface ProductImagesModalProps {
  productId: number;
  productName?: string;
  onClose: () => void;
}

export default function ProductImagesModal({
  productId,
  productName,
  onClose,
}: ProductImagesModalProps) {
  return (
    <div className="fixed inset-0 z-99999 flex items-center justify-center bg-black/50 p-4 backdrop-blur-xs">
      <div className="max-h-[90vh] w-full max-w-5xl overflow-y-auto rounded-2xl bg-white p-6 shadow-2xl dark:bg-gray-900">
        <div className="mb-5 flex items-center justify-between border-b border-gray-100 pb-4 dark:border-gray-800">
          <div>
            <h2 className="text-lg font-semibold text-gray-800 dark:text-white">
              Manage Photos {productName ? `— ${productName}` : ""}
            </h2>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              Product ID #{productId}
            </p>
          </div>
          <button
            onClick={onClose}
            aria-label="Close"
            className="rounded-lg p-1.5 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-700 dark:hover:bg-gray-800 dark:hover:text-gray-200"
          >
            <X size={20} />
          </button>
        </div>

        <ProductImagesManager productId={productId} />
      </div>
    </div>
  );
}

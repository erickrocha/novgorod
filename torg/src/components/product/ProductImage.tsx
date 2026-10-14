import { ImageOff } from 'lucide-react';

interface Props { src?: string; alt: string; className?: string }

/** The product photo, or a neutral placeholder when the product has none (no stock image is invented). */
export default function ProductImage({ src, alt, className = '' }: Props) {
  if (!src) {
    return (
      <div role="img" aria-label={`${alt} (sem imagem)`} className={`flex items-center justify-center bg-slate-100 text-slate-300 ${className}`}>
        <ImageOff className="w-1/3 h-1/3 max-w-10 max-h-10" aria-hidden="true" />
      </div>
    );
  }
  return <img src={src} alt={alt} className={className} loading="lazy" />;
}

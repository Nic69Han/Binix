# Architecture Binix

## Crates
- binix-core : Types, erreurs, traits
- binix-net : Réseau & cache
- binix-dom : Parser HTML
- binix-css : CSS & styles
- binix-layout : Layout (taffy)
- binix-compositor : GPU rendering
- binix-js : JavaScript (Boa)
- binix-security : Sécurité & CSP
- binix-ui : Interface egui
- binix-app : Application principale

## Flux de données
URL → Network → DOM → CSS → Layout → Paint → GPU
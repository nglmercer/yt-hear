# GitHub Actions Workflows

Este directorio contiene los workflows de GitHub Actions para el proyecto Tatar.

## Workflows Disponibles

### 1. Release (`release.yml`)

**Trigger**: Se ejecuta cuando se crea un tag que empieza con `v*` (ej: `v1.0.0`, `v1.2.3`)

**Propósito**: Build y release automático de la aplicación Tauri para Linux y Windows.

**Características**:
- Build multiplataforma (Ubuntu y Windows)
- Genera artefactos para ambas plataformas:
  - **Windows**: MSI installer
  - **Linux**: DEB package y AppImage
- Crea un release en GitHub con todos los artefactos
- Incluye instrucciones de instalación
- Usa cache para optimizar tiempos de build

**Uso**:
```bash
# Crear un nuevo release
git tag v1.0.0
git push origin v1.0.0
```

### 2. CI (`ci.yml`)

**Trigger**: Se ejecuta en cada push y pull request a las ramas `main` y `develop`

**Propósito**: Integración continua para asegurar que el código siempre compile y funcione.

**Características**:
- Test en múltiples plataformas (Ubuntu y Windows)
- Verificación de build del frontend
- Verificación de compilación de Rust
- Ejecución de tests (si existen)
- Build de Tauri en modo debug para verificar que todo funciona

### 3. Lint (`lint.yml`)

**Trigger**: Se ejecuta en cada push y pull request a las ramas `main` y `develop`

**Propósito**: Asegurar la calidad y consistencia del código.

**Características**:
- **Frontend**: Verificación de linting (si está configurado)
- **Backend**: 
  - Formato con `rustfmt`
  - Linting con `clippy`
- Falla el build si hay problemas de formato o advertencias de clippy

## Configuración Requerida

### Secrets de GitHub
No se requieren secrets adicionales. Los workflows usan:
- `GITHUB_TOKEN`: Token automático de GitHub Actions para crear releases

### Dependencias del Proyecto
Asegúrate de que tu proyecto tenga:
- `package.json` con scripts de build y test
- `src-tauri/tauri.conf.json` configurado correctamente
- `src-tauri/Cargo.toml` con las dependencias de Tauri

## Configuración Adicional Recomendada

### 1. Protección de Branches
Configura protección para la rama `main`:
- Requerir revisiones de código
- Requerir que pasen los checks de CI y Lint
- Impedir force pushes

### 2. Scripts de Test
Agrega tests a tu proyecto:

**Frontend** (package.json):
```json
{
  "scripts": {
    "test": "vitest",
    "lint": "eslint src --ext .js,.vue"
  }
}
```

**Backend** (src-tauri/Cargo.toml):
```toml
[dev-dependencies]
# Tests van aquí
```

### 3. Linting de Frontend
Instala y configura ESLint para tu frontend:
```bash
npm install --save-dev eslint eslint-plugin-vue
```

## Flujo de Trabajo Recomendado

1. **Desarrollo**: Trabaja en feature branches
2. **Pull Request**: Abre PR hacia `develop` o `main`
3. **Validación Automática**: CI y Lint se ejecutan automáticamente
4. **Merge**: Solo después de que todos los checks pasen
5. **Release**: Crea un tag cuando quieras publicar una nueva versión

## Troubleshooting

### Build Falla en Linux
Asegúrate de que las dependencias del sistema están correctamente instaladas. El workflow ya instala las dependencias necesarias para Tauri en Ubuntu.

### Release No se Crea
Verifica que:
- El tag tenga el formato correcto (`v*`)
- Tengas permisos para crear releases en el repositorio
- El `GITHUB_TOKEN` tenga los permisos necesarios

### Linting Falla
Ejecuta localmente los comandos de linting:
```bash
# Frontend
npm run lint

# Backend
cd src-tauri
cargo fmt --all
cargo clippy --all-targets --all-features
```

## Personalización

Puedes personalizar los workflows según necesites:

- Agregar más plataformas (macOS)
- Modificar las versiones de Node.js o Rust
- Agregar pasos de testing adicionales
- Configurar notificaciones
- Agregar deployments automáticos

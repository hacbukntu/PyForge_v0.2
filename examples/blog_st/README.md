# Blog ST: lógica de dominio compilable

```bash
pyforgec build examples/blog_st/main.st -o /tmp/blog-st
/tmp/blog-st
```

`modelos.st` valida el título, `vistas.st` compone el texto y `urls.st` contiene
la ruta. Este ejemplo no arranca HTTP: muestra la parte ST admitida hoy. Los tres
proyectos HTTP completos están en `examples/web`. La API Rust experimental y lo
que falta para `importar "django"` se explican en `docs/INTEGRACIONES.md`.

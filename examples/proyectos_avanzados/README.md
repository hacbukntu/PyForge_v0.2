# 20 proyectos profesionales y juegos ST

Colección independiente de los 80 proyectos anteriores y de los 12 ejemplos de integración.
Son aplicaciones didácticas completas dentro de los límites actuales, no productos certificados para producción.

| Proyecto | Categoría | Módulos | Ejecución |
|---|---|---|---|
| [Caja de tienda con recibo JSON](01_caja_de_tienda/README.md) | Negocio | db, json, io | local |
| [Informe comercial Word y PDF](02_informe_comercial/README.md) | Documentos | documentos, archivos, json | documentos |
| [Inventario con movimientos transaccionales](03_inventario_transaccional/README.md) | Negocio | db, json | local |
| [Agenda y exportación CSV](04_agenda_csv/README.md) | Productividad | archivos, io, json | local |
| [Diario local con cifrado autenticado](05_diario_cifrado/README.md) | Seguridad | cifrado, archivos, io, sistema | local |
| [Verificador de integridad de entregables](06_verificador_integridad/README.md) | Seguridad | cifrado, archivos, json, os | local |
| [Respaldo versionado de configuración](07_respaldo_configuracion/README.md) | Administración | archivos, json, time, os | local |
| [Planificador de ahorro mensual](08_presupuesto_personal/README.md) | Finanzas educativas | math, io | local |
| [Temporizador de enfoque con registro](09_temporizador_enfoque/README.md) | Productividad | time, io, archivos, json | local |
| [Juego de reto matemático por rondas](10_reto_matematico/README.md) | Juego | math, time, io, json | local |
| [Ahorcado de palabras Unicode](11_ahorcado_unicode/README.md) | Juego | io, json | local |
| [Batalla naval de bolsillo](12_batalla_naval/README.md) | Juego | io, json | local |
| [Laberinto con paredes y salida](13_laberinto_terminal/README.md) | Juego | io | local |
| [Duelo de héroes con clases](14_duelo_por_turnos/README.md) | Juego | io, json | local |
| [Juego de memoria de secuencias](15_memoria_secuencias/README.md) | Juego | time, io | local |
| [Servicio de consultas por TCP](16_consultas_tcp/README.md) | Redes | red.tcp, red.socket, sistema, cifrado | tcp |
| [Telemetría UDP con identificador de mensaje](17_telemetria_udp/README.md) | Redes | red.udp, json, cifrado, sistema | udp |
| [Cliente de estado HTTPS con CA verificada](18_cliente_https/README.md) | Redes | red.ssl, cifrado, sistema | tls |
| [Entrega de archivos mediante SFTP o FTPS](19_transferencias_seguras/README.md) | Redes | red.ssh, red.sftp, red.ftp, sistema, archivos, io | remoto |
| [Panel gráfico de operaciones locales](20_panel_operaciones/README.md) | Interfaz gráfica | gui, integraciones, vpn, comandos, sistema, json, time | gui |

## Ejecutar y comprobar

Entra en la carpeta del proyecto y usa `pyforge run`. En Windows, usa WSL.
Revisa primero su README, permisos y archivos que genera. Para repetir sin conflictos, usa una copia nueva.

```bash
python3 scripts/verify-advanced-projects.py
```

El verificador compila todos, ejecuta los autónomos y laboratorios locales cuando puede, y declara omisiones.
Usa `--python /ruta/absoluta/al/python` para dependencias en un entorno virtual y `--strict` para fallar ante cualquier omisión.
No se conecta a servicios externos. Los puertos, certificados y credenciales de laboratorio son temporales.

Cobertura: time, os, math, io, json, integraciones, archivos, db, documentos, red, cifrado, comandos, sistema, vpn y gui.
Los alias TCP/UDP/TLS/SSH/SFTP/FTPS corresponden a las APIs implementadas de red.
VPN solo consulta información local. La rama FTPS de transferencias requiere un servidor de laboratorio y verificación adicional.

Resultados y límites: [docs/MANUAL_ST.md](../../docs/MANUAL_ST.md).

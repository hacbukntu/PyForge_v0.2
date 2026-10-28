# 80 proyectos con ST

Colección de proyectos independientes, comentados en español y ejecutables con PyForge nativo. Cada carpeta contiene código, instrucciones y un caso de prueba con salida esperada. Son ejemplos funcionales acotados: consulta los límites de cada proyecto antes de adaptarlo.

## Comenzar

Desde la carpeta de cualquier proyecto:

```bash
pyforge run
# O bien:
pyforgec run src/main.st
```

En el IDE abre la carpeta del proyecto, selecciona `src/main.st` y pulsa Ejecutar. Responde en la terminal inferior. Los archivos de salida se escriben en la carpeta de ejecución. **Ejecuta desde la carpeta del proyecto** para que se encuentren los datos de ejemplo.

## Comprobar la colección

Desde la raíz del repositorio: `python3 scripts/verify-projects.py`. El verificador compila con y sin optimización y ejecuta en copias temporales, con entrada predefinida, tiempo máximo y comparación de salida y archivos. No modifica los proyectos originales. Python solo se usa para las pruebas; los programas están escritos en ST y se ejecutan como binarios nativos.

## Catálogo

| N.º | Proyecto | Área | Qué hace |
| --- | --- | --- | --- |
| 01 | [Adivina el número](01_adivina_numero/README.md) | Juegos | Practica búsqueda por pistas con un máximo de cinco intentos. |
| 02 | [Piedra, papel y tijera](02_piedra_papel_tijera/README.md) | Juegos | Juega tres rondas contra una secuencia de movimientos reproducible. |
| 03 | [Tres en raya para dos jugadores](03_tres_en_raya/README.md) | Juegos | Alterna jugadores, valida casillas y detecta filas, columnas y diagonales. |
| 04 | [Ahorcado](04_ahorcado/README.md) | Juegos | Descubre una palabra conservando letras acertadas y controlando seis fallos. |
| 05 | [Carrera de dados](05_carrera_dados/README.md) | Juegos | Simula una carrera con semilla elegida por el usuario y resultados reproducibles. |
| 06 | [Nim: retirar fichas](06_nim/README.md) | Juegos | Juega contra una estrategia que deja múltiplos de cuatro. |
| 07 | [Aventura de decisiones](07_aventura/README.md) | Juegos | Explora dos rutas y combina una llave con una decisión final. |
| 08 | [Trivia con puntuación](08_trivia/README.md) | Juegos | Corrige respuestas, calcula puntos y presenta el porcentaje de aciertos. |
| 09 | [Laberinto de cuadrícula](09_laberinto/README.md) | Juegos | Navega con WASD, respeta paredes y llega a una salida. |
| 10 | [Combate por turnos](10_combate_turnos/README.md) | Juegos | Administra salud, pociones y ataques en una batalla de consola. |
| 11 | [Factura en centavos](11_factura/README.md) | Negocio | Calcula subtotal, impuesto configurable y total evitando redondeo binario. |
| 12 | [Descuentos por volumen](12_descuentos/README.md) | Negocio | Aplica descuentos escalonados y obtiene el importe final en centavos. |
| 13 | [Repartir una cuenta](13_dividir_cuenta/README.md) | Negocio | Distribuye centavos y asigna el resto sin perder dinero en el reparto. |
| 14 | [Control de presupuesto](14_presupuesto/README.md) | Negocio | Clasifica gastos y avisa si una categoría supera su presupuesto. |
| 15 | [Meta de ahorro](15_meta_ahorro/README.md) | Negocio | Estima cuántos meses hacen falta con aportaciones constantes, sin intereses. |
| 16 | [Punto de equilibrio](16_punto_equilibrio/README.md) | Negocio | Calcula unidades necesarias para cubrir costes fijos con un margen positivo. |
| 17 | [Plan de cuotas sin intereses](17_cuotas/README.md) | Negocio | Genera un calendario de cuotas iguales y ajusta la última por el resto. |
| 18 | [Cierre de caja](18_caja_diaria/README.md) | Negocio | Reconcilia apertura, ventas, devoluciones y efectivo contado. |
| 19 | [Valor de inventario](19_valor_inventario/README.md) | Negocio | Calcula el coste total del almacén y detecta existencias bajas. |
| 20 | [Cotizador de servicios](20_cotizador_servicios/README.md) | Negocio | Combina horas, tarifa, materiales y recargo por urgencia. |
| 21 | [Lista de tareas](21_lista_tareas/README.md) | Organización | Gestiona tareas pendientes y marca una como terminada mediante su índice. |
| 22 | [Agenda de contactos](22_agenda/README.md) | Organización | Busca teléfonos por nombre sin consultar claves inexistentes. |
| 23 | [Plan de estudio](23_plan_estudio/README.md) | Organización | Distribuye temas en días según un presupuesto de minutos. |
| 24 | [Planificador de bloques de concentración](24_pomodoro/README.md) | Organización | Genera una secuencia de trabajo y descansos sin bloquear minutos durante la demostración. |
| 25 | [Registro semanal de hábitos](25_habitos/README.md) | Organización | Cuenta días cumplidos y la mejor racha consecutiva. |
| 26 | [Detector de reuniones superpuestas](26_reuniones/README.md) | Organización | Compara intervalos de minutos y detecta conflictos de agenda. |
| 27 | [Rotación de turnos](27_turnos/README.md) | Organización | Asigna personal por rotación circular para evitar una asignación fija. |
| 28 | [Lista de equipaje por peso](28_mochila/README.md) | Organización | Selecciona elementos prioritarios que caben en un límite de peso. |
| 29 | [Lista de compras consolidada](29_lista_compras/README.md) | Organización | Acumula productos repetidos conservando el orden de primera aparición. |
| 30 | [Cola de prioridades de tareas](30_prioridades/README.md) | Organización | Ordena tareas por urgencia usando listas paralelas e intercambio estable. |
| 31 | [Contador de texto](31_contador_texto/README.md) | Texto y datos | Cuenta caracteres Unicode, palabras y líneas de un texto de entrada. |
| 32 | [Verificador de palíndromos](32_palindromo/README.md) | Texto y datos | Compara caracteres extremos ignorando espacios y signos elegidos. |
| 33 | [Generador de identificadores URL](33_slug/README.md) | Texto y datos | Transforma títulos ASCII a minúsculas con guiones y elimina símbolos. |
| 34 | [Ocultar datos en un texto](34_censurar_datos/README.md) | Texto y datos | Sustituye un identificador conocido y una dirección de ejemplo antes de compartir un mensaje. |
| 35 | [Buscar coincidencias solapadas](35_buscar_ocurrencias/README.md) | Texto y datos | Encuentra todas las posiciones donde aparece un patrón corto. |
| 36 | [Frecuencia de caracteres](36_frecuencia_letras/README.md) | Texto y datos | Agrupa caracteres Unicode y mantiene su orden de aparición. |
| 37 | [Lector de CSV sencillo](37_csv_simple/README.md) | Texto y datos | Separa una fila en campos conservando los vacíos. |
| 38 | [Compresor RLE de texto](38_rle/README.md) | Texto y datos | Codifica secuencias de caracteres iguales como símbolo y cantidad. |
| 39 | [Estadísticas descriptivas](39_estadisticas/README.md) | Texto y datos | Calcula media, mínimo, máximo y amplitud de una serie. |
| 40 | [Media móvil de ventas](40_media_movil/README.md) | Texto y datos | Suaviza una serie usando una ventana deslizante de tres muestras. |
| 41 | [Bloc de notas persistente](41_bloc_notas/README.md) | Archivos y JSON | Lee una nota desde la terminal, la guarda y verifica su contenido. |
| 42 | [Copia verificada de texto](42_copia_texto/README.md) | Archivos y JSON | Copia un archivo UTF-8 y compara origen y destino. |
| 43 | [Diario con conservación de entradas](43_diario/README.md) | Archivos y JSON | Añade una entrada conservando el contenido previo mediante lectura y reescritura. |
| 44 | [Configuración JSON tipada](44_configuracion_json/README.md) | Archivos y JSON | Carga opciones de aplicación y convierte explícitamente sus valores. |
| 45 | [Exportar formulario a JSON](45_formulario_json/README.md) | Archivos y JSON | Valida edad y serializa nombre, edad y consentimiento. |
| 46 | [Consultar catálogo JSON](46_catalogo_json/README.md) | Archivos y JSON | Recorre registros anidados y filtra productos disponibles. |
| 47 | [Validador de documentos JSON](47_validar_json/README.md) | Archivos y JSON | Informa del primer problema de sintaxis mediante excepciones capturables. |
| 48 | [Combinar configuraciones conocidas](48_fusion_config/README.md) | Archivos y JSON | Combina un nombre base y un límite local, validando las claves esperadas. |
| 49 | [Limpiar líneas vacías](49_limpiar_lineas/README.md) | Archivos y JSON | Normaliza CRLF y elimina líneas completamente vacías de un archivo. |
| 50 | [Resumen de registros de aplicación](50_analizador_logs/README.md) | Archivos y JSON | Cuenta mensajes por nivel a partir de prefijos de líneas. |
| 51 | [Ordenación por inserción](51_ordenar/README.md) | Algoritmos | Ordena una lista manteniendo los elementos de igual valor en su orden relativo. |
| 52 | [Búsqueda binaria](52_busqueda_binaria/README.md) | Algoritmos | Localiza un elemento en una colección ordenada con intervalo semiabierto. |
| 53 | [Criba de números primos](53_primos/README.md) | Algoritmos | Marca múltiplos para enumerar primos hasta un límite pequeño. |
| 54 | [Máximo común divisor y mínimo común múltiplo](54_mcd_mcm/README.md) | Algoritmos | Utiliza el algoritmo de Euclides con comprobación de entradas positivas. |
| 55 | [Serie de Fibonacci iterativa](55_fibonacci/README.md) | Algoritmos | Calcula una serie con memoria constante y limita el índice al rango i64. |
| 56 | [Cambio mínimo con programación dinámica](56_cambio_monedas/README.md) | Algoritmos | Obtiene el menor número de monedas para un importe alcanzable. |
| 57 | [Ruta mínima en un grafo](57_ruta_bfs/README.md) | Algoritmos | Recorre un grafo no ponderado con cola e identifica distancias. |
| 58 | [Multiplicación de matrices](58_matrices/README.md) | Algoritmos | Multiplica dos matrices 2x2 usando almacenamiento plano. |
| 59 | [Operaciones entre conjuntos](59_conjuntos/README.md) | Algoritmos | Calcula unión e intersección eliminando repetidos con conjuntos nativos. |
| 60 | [Orden de dependencias](60_orden_topologico/README.md) | Algoritmos | Planifica tareas de un grafo acíclico y detecta ciclos. |
| 61 | [Cuenta con saldo controlado](61_cuenta_saldo/README.md) | Modelado con clases | Modela depósitos y retiros con validación de saldo y errores capturables. |
| 62 | [Préstamos de biblioteca](62_biblioteca_libros/README.md) | Modelado con clases | Gestiona el préstamo y devolución de un libro con transiciones válidas. |
| 63 | [Carrito de compra](63_carrito/README.md) | Modelado con clases | Encapsula cantidades y calcula un subtotal al agregar productos. |
| 64 | [Estado de un pedido](64_pedido_estados/README.md) | Modelado con clases | Aplica una máquina de estados para pagar, enviar y entregar pedidos. |
| 65 | [Reservas con aforo](65_reservas/README.md) | Modelado con clases | Reserva y libera plazas sin superar un aforo fijo. |
| 66 | [Cronómetro monotónico](66_cronometro/README.md) | Modelado con clases | Mide una pausa corta y encapsula el instante inicial en un objeto. |
| 67 | [Sensor con umbral](67_sensor/README.md) | Modelado con clases | Valida lecturas y emite un estado según límites configurables. |
| 68 | [Geometría de rectángulos](68_rectangulos/README.md) | Modelado con clases | Usa estructura y métodos agregados con implementar para calcular área y perímetro. |
| 69 | [Pila de tareas](69_pila/README.md) | Modelado con clases | Implementa apilar y desapilar con un contador lógico sobre una lista. |
| 70 | [Calculadora modular](70_calculadora_modular/README.md) | Modelado con clases | Separa reglas aritméticas del programa de consola mediante importaciones locales. |
| 71 | [Portafolio HTML](71_portafolio_html/README.md) | Informes y documentos | Genera una página personal con proyectos escapando el nombre del autor. |
| 72 | [Catálogo HTML](72_catalogo_html/README.md) | Informes y documentos | Crea tarjetas de productos con etiquetas de disponibilidad. |
| 73 | [Informe HTML de factura](73_factura_html/README.md) | Informes y documentos | Transforma líneas de una factura en una tabla con total calculado. |
| 74 | [Panel HTML de actividad](74_panel_barras/README.md) | Informes y documentos | Muestra barras proporcionales y datos accesibles como texto. |
| 75 | [Exportador CSV con comillas](75_exportar_csv/README.md) | Informes y documentos | Escapa comillas y separadores al exportar nombres y cantidades. |
| 76 | [Informe de avances Markdown](76_informe_markdown/README.md) | Informes y documentos | Produce una tabla de seguimiento con porcentajes calculados. |
| 77 | [Exportar evento de calendario](77_agenda_ics/README.md) | Informes y documentos | Genera un evento iCalendar de ejemplo con horas UTC explícitas. |
| 78 | [Tarjeta de contacto vCard](78_contacto_vcard/README.md) | Informes y documentos | Exporta un contacto ficticio como tarjeta de texto estructurado. |
| 79 | [Canal RSS estático](79_canal_rss/README.md) | Informes y documentos | Construye un documento XML con títulos escapados y enlaces de ejemplo. |
| 80 | [Cronograma SVG](80_gantt_svg/README.md) | Informes y documentos | Convierte duraciones y posiciones de tareas en un gráfico vectorial autónomo. |

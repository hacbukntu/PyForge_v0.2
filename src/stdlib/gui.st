// MAPA DE CONEXIÓN DEL MÓDULO
// Programa .st -> estas funciones -> integraciones.pedir -> auxiliar Python.
// runtime/integraciones.py comprueba permisos y administra GuiSession.
// runtime/gui.py construye Qt y devuelve eventos JSON por pipes locales.
// Para agregar una API: añade aquí una función pública, registra gui.nombre en
// Runtime.operations y crea su método en runtime/integraciones.py; si modifica
// controles, añade la orden correspondiente en receive() de runtime/gui.py.
// Para agregar un control visual: amplía build() en runtime/gui.py.
// Para cambiar colores y tamaño: main() en runtime/gui.py (resize/setStyleSheet).
// La documentación paso a paso está en docs/INTEGRACIONES.md.
// El IDE descubre las firmas del módulo; recompila para ejecutar cambios nuevos.
// GUI opcional PySide6: proceso dedicado con Qt en su hilo principal.
// El diseño es un objeto JSON; los eventos se consultan sin bloquear la interfaz.
importar "integraciones";
importar "json";
// Crea una ventana declarativa; devuelve un identificador de sesión.
funcion crear(diseno: cadena) -> entero {
    // Valida sintaxis JSON antes de enviarlo; crear devuelve un recurso, no un puntero Qt.
    json.comprobar(diseno);
    retornar json.a_entero(integraciones.pedir("gui.crear", "[" + diseno + "]"));
}
// Retorna un objeto JSON con evento: lista, click, texto, cambio, seleccion, cerrada o ninguno.
funcion evento(ventana: entero) -> cadena {
    // No bloquea: "ninguno" indica que el programa debe seguir o dormir brevemente.
    retornar integraciones.pedir("gui.evento", "[" + json.de_entero(ventana) + "]");
}
// Cambia el texto de etiqueta, botón o entrada; los fallos llegan como eventos.
funcion cambiar_texto(ventana: entero, control: cadena, texto: cadena) -> booleano {
    // Codifica identificador y texto como datos; true confirma envío, no aplicación.
    retornar json.a_booleano(integraciones.pedir("gui.texto", "[" + json.de_entero(ventana) + "," + json.codificar(control) + "," + json.codificar(texto) + "]"));
}
// Cierra el proceso gráfico propio y libera sus descriptores.
funcion cerrar(ventana: entero) -> booleano {
    // Usa finalmente en ST para liberar también cuando una operación lance un error.
    retornar json.a_booleano(integraciones.pedir("gui.cerrar", "[" + json.de_entero(ventana) + "]"));
}

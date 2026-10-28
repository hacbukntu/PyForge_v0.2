// Tiempo nativo Linux; entero i64, unidades explícitas para evitar conversiones ambiguas.
// Milisegundos desde el epoch Unix; el reloj del sistema puede ajustarse.
funcion ahora() -> entero { 
    retornar aux_reloj(0); 
    }
// Milisegundos monotónicos: usar diferencias para medir duraciones, no fechas.
funcion monotono() -> entero { 
    retornar aux_reloj(1); 
    }
// Espera al menos la duración indicada, salvo error del sistema; no ocupa un bucle activo.
funcion dormir(milisegundos: entero) -> nada {
    si milisegundos < 0 { 
        lanzar "time.dormir: la duración no puede ser negativa"; 
        }
    si aux_dormir(milisegundos) < 0 { 
        lanzar "time.dormir: falló nanosleep"; 
        }
}

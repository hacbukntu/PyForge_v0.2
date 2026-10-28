// JSON nativo expresado en ST. Los valores compuestos se conservan como texto JSON,
// evitando inventar un tipo dinámico incompatible con las colecciones homogéneas.
// Límites explícitos: 4096 caracteres por documento y 64 niveles de anidación.

// Comprueba tamaño antes de recorrer o reservar nuevas cadenas.
funcion aux_limite(texto: cadena) -> nada {
    si longitud(texto) > 4096 { lanzar "json: máximo 4096 caracteres por documento"; }
}
// Acceso seguro: el final del documento se representa mediante cadena vacía.
funcion aux_letra(texto: cadena, posicion: entero) -> cadena {
    si posicion >= longitud(texto) { retornar ""; }
    retornar texto[posicion];
}
// Ignora únicamente los cuatro espacios permitidos por JSON.
funcion aux_espacios(texto: cadena, posicion: entero) -> entero {
    variable i: entero = posicion;
    variable c: cadena = aux_letra(texto, i);
    mientras c == " " o c == "\t" o c == "\r" o c == "\n" {
        i = i + 1;
        c = aux_letra(texto, i);
       
    }
    retornar i;
}
// Devuelve el valor hexadecimal o -1, sin conversiones numéricas implícitas.
funcion aux_hex(c: cadena) -> entero {
    si c == "" { retornar -1; }
    variable indice: entero = buscar("0123456789abcdef", c);
    si indice >= 0 { retornar indice; }
    retornar buscar("0123456789ABCDEF", c);
}
// Lee exactamente cuatro dígitos de un escape UTF-16.
funcion aux_unidad(texto: cadena, inicio: entero) -> entero {
    variable valor: entero = 0;
    para i en rango(inicio, inicio + 4) {
        variable digito: entero = aux_hex(aux_letra(texto, i));
        si digito < 0 { lanzar "json: escape Unicode incompleto o inválido"; }
        valor = valor * 16 + digito;
    }
    retornar valor;
}
// Devuelve el escalar Unicode; une pares sustitutos y rechaza sustitutos aislados.
funcion aux_unicode(texto: cadena, inicio: entero) -> entero {
    variable valor: entero = aux_unidad(texto, inicio);
    variable bajo: entero = 0;
    si valor >= 55296 y valor <= 56319 {
        si aux_letra(texto, inicio + 4) != "\\" o aux_letra(texto, inicio + 5) != "u" {
            lanzar "json: falta el sustituto bajo Unicode";
        }
        bajo = aux_unidad(texto, inicio + 6);
        si bajo < 56320 o bajo > 57343 { lanzar "json: sustituto bajo inválido"; }
        retornar 65536 + (valor - 55296) * 1024 + bajo - 56320;
    }
    si valor >= 56320 y valor <= 57343 { lanzar "json: sustituto Unicode aislado"; }
    retornar valor;
}
// Avanza sobre una cadena completa, validando controles, escapes y cierre.
funcion aux_cadena_fin(texto: cadena, inicio: entero) -> entero {
    si aux_letra(texto, inicio) != "\"" { lanzar "json: se esperaba una cadena"; }
    variable i: entero = inicio + 1;
    variable punto: entero = 0;
    mientras i < longitud(texto) {
        variable c: cadena = texto[i];
        si c == "\"" { retornar i + 1; }
        si aux_codigo(c) < 32 { lanzar "json: carácter de control sin escape"; }
        si c == "\\" {
            i = i + 1;
            c = aux_letra(texto, i);
            si c == "u" {
                punto = aux_unicode(texto, i + 1);
                i = i + 4;
                si punto > 65535 { i = i + 6; }
            } sino {
                si c == "" o buscar("\"\\/bfnrt", c) < 0 { lanzar "json: escape inválido"; }
            }
        }
        i = i + 1;
    }
    lanzar "json: cadena sin cerrar";
    retornar i;
}
// Predicado ASCII: JSON no admite dígitos de otros alfabetos.
funcion aux_digito(c: cadena) -> booleano { retornar c != "" y buscar("0123456789", c) >= 0; }
// Recorre la gramática numérica sin perder precisión convirtiéndola a f64.
funcion aux_numero_fin(texto: cadena, inicio: entero) -> entero {
    variable i: entero = inicio;
    si aux_letra(texto, i) == "-" { i = i + 1; }
    si aux_letra(texto, i) == "0" { i = i + 1; }
    sino {
        si no aux_digito(aux_letra(texto, i)) { lanzar "json: número inválido"; }
        mientras aux_digito(aux_letra(texto, i)) { i = i + 1; }
    }
    si aux_letra(texto, i) == "." {
        i = i + 1;
        si no aux_digito(aux_letra(texto, i)) { lanzar "json: faltan decimales"; }
        mientras aux_digito(aux_letra(texto, i)) { i = i + 1; }
    }
    variable c: cadena = aux_letra(texto, i);
    si c == "e" o c == "E" {
        i = i + 1;
        c = aux_letra(texto, i);
        si c == "+" o c == "-" { i = i + 1; }
        si no aux_digito(aux_letra(texto, i)) { lanzar "json: exponente incompleto"; }
        mientras aux_digito(aux_letra(texto, i)) { i = i + 1; }
    }
    retornar i;
}
// Extrae un intervalo de caracteres ya comprobado por el analizador.
funcion aux_fragmento(texto: cadena, inicio: entero, fin: entero) -> cadena {
    variable salida: cadena = "";
    para i en rango(inicio, fin) { salida = salida + texto[i]; }
    retornar salida;
}
// Analizador recursivo: retorna la primera posición posterior al valor, sin consumir comas.
funcion aux_valor_fin(texto: cadena, inicio: entero, profundidad: entero) -> entero {
    variable i: entero = aux_espacios(texto, inicio);
    variable c: cadena = aux_letra(texto, i);
    variable objeto: booleano = falso;
    variable cierre: cadena = "]";
    variable literal: cadena = "null";
    variable j: entero = 0;
    si c == "\"" { retornar aux_cadena_fin(texto, i); }
    si c == "[" o c == "{" {
        si profundidad >= 64 { lanzar "json: máximo 64 niveles de anidación"; }
        objeto = c == "{";
        cierre = "]";
        si objeto { cierre = "}"; }
        i = aux_espacios(texto, i + 1);
        si aux_letra(texto, i) == cierre { retornar i + 1; }
        mientras verdadero {
            si objeto {
                i = aux_espacios(texto, aux_cadena_fin(texto, i));
                si aux_letra(texto, i) != ":" { lanzar "json: falta ':' después de la clave"; }
                i = i + 1;
            }
            i = aux_espacios(texto, aux_valor_fin(texto, i, profundidad + 1));
            c = aux_letra(texto, i);
            si c == cierre { retornar i + 1; }
            si c != "," { lanzar "json: se esperaba coma o cierre"; }
            i = aux_espacios(texto, i + 1);
        }
    }
    si c == "t" o c == "f" o c == "n" {
        literal = "null";
        si c == "t" { literal = "true"; }
        si c == "f" { literal = "false"; }
        para j en rango(0, longitud(literal)) {
            si aux_letra(texto, i + j) != literal[j] { lanzar "json: literal inválido"; }
        }
        retornar i + longitud(literal);
    }
    retornar aux_numero_fin(texto, i);
}
// Exige un único valor completo; los errores pueden capturarse mediante intentar/capturar.
funcion comprobar(texto: cadena) -> nada {
    aux_limite(texto);
    variable fin: entero = aux_espacios(texto, aux_valor_fin(texto, 0, 0));
    si fin != longitud(texto) { lanzar "json: contenido adicional después del valor"; }
}
// Variante booleana para entrada externa: captura exclusivamente las excepciones ST.
funcion validar(texto: cadena) -> booleano {
    intentar { comprobar(texto); retornar verdadero; }
    capturar error { retornar falso; }
    retornar falso;
}
// Serializa una cadena ST: escapa comillas, barras y todos los controles U+0000..U+001F.
funcion codificar(texto: cadena) -> cadena {
    aux_limite(texto);
    variable salida: cadena = "\"";
    para i en rango(0, longitud(texto)) {
        variable c: cadena = texto[i];
        variable codigo: entero = aux_codigo(c);
        si c == "\"" o c == "\\" { salida = salida + "\\" + c; }
        sino_si codigo < 32 {
            salida = salida + "\\u00" + "0123456789abcdef"[codigo / 16] + "0123456789abcdef"[codigo % 16];
        } sino { salida = salida + c; }
        // Detiene la expansión de controles antes de construir un documento excesivo.
        si longitud(salida) >= 4096 { lanzar "json: máximo 4096 caracteres por documento"; }
    }
    salida = salida + "\"";
    aux_limite(salida);
    retornar salida;
}
// Decodifica exclusivamente un valor JSON de tipo cadena, incluyendo escapes UTF-16.
funcion decodificar(texto: cadena) -> cadena {
    comprobar(texto);
    variable inicio: entero = aux_espacios(texto, 0);
    variable fin: entero = aux_cadena_fin(texto, inicio) - 1;
    variable punto: entero = 0;
    variable i: entero = inicio + 1;
    variable salida: cadena = "";
    mientras i < fin {
        variable c: cadena = texto[i];
        si c == "\\" {
            i = i + 1;
            c = texto[i];
            si c == "u" {
                punto = aux_unicode(texto, i + 1);
                c = aux_caracter(punto);
                i = i + 4;
                si punto > 65535 { i = i + 6; }
            } sino_si c == "b" { c = aux_caracter(8); }
            sino_si c == "f" { c = aux_caracter(12); }
            sino_si c == "n" { c = "\n"; }
            sino_si c == "r" { c = "\r"; }
            sino_si c == "t" { c = "\t"; }
        }
        salida = salida + c;
        i = i + 1;
    }
    retornar salida;
}
// Busca una clave decodificada y devuelve su valor como texto JSON validado.
// Si la clave se repite se devuelve la primera; la ausencia produce excepción.
funcion obtener(texto: cadena, clave: cadena) -> cadena {
    comprobar(texto);
    variable i: entero = aux_espacios(texto, 0);
    si aux_letra(texto, i) != "{" { lanzar "json.obtener: se requiere un objeto"; }
    i = aux_espacios(texto, i + 1);
    mientras aux_letra(texto, i) != "}" {
        variable fin_clave: entero = aux_cadena_fin(texto, i);
        variable nombre: cadena = decodificar(aux_fragmento(texto, i, fin_clave));
        i = aux_espacios(texto, aux_espacios(texto, fin_clave) + 1);
        variable fin: entero = aux_valor_fin(texto, i, 0);
        si nombre == clave { retornar aux_fragmento(texto, i, fin); }
        i = aux_espacios(texto, fin);
        si aux_letra(texto, i) == "," { i = aux_espacios(texto, i + 1); }
    }
    lanzar "json.obtener: clave no encontrada: " + clave;
    retornar "";
}
// Obtiene un elemento por índice no negativo y conserva objetos/listas anidados como JSON.
funcion elemento(texto: cadena, indice: entero) -> cadena {
    comprobar(texto);
    si indice < 0 { lanzar "json.elemento: índice negativo"; }
    variable i: entero = aux_espacios(texto, 0);
    si aux_letra(texto, i) != "[" { lanzar "json.elemento: se requiere un arreglo"; }
    i = aux_espacios(texto, i + 1);
    variable actual: entero = 0;
    mientras aux_letra(texto, i) != "]" {
        variable fin: entero = aux_valor_fin(texto, i, 0);
        si actual == indice { retornar aux_fragmento(texto, i, fin); }
        actual = actual + 1;
        i = aux_espacios(texto, fin);
        si aux_letra(texto, i) == "," { i = aux_espacios(texto, i + 1); }
    }
    lanzar "json.elemento: índice fuera de rango";
    retornar "";
}
// Serialización exacta de i64: no pasa por flotante ni pierde dígitos.
funcion de_entero(valor: entero) -> cadena { retornar cadena(valor); }
// JSON usa true/false en minúscula, distintos del formato de mostrar(booleano).
funcion de_booleano(valor: booleano) -> cadena { si valor { retornar "true"; } retornar "false"; }
// JSON null es texto de intercambio, no el literal ST nulo.
funcion valor_nulo() -> cadena { retornar "null"; }

// Identifica el tipo JSON sin convertir objetos a colecciones ST heterogéneas.
funcion tipo(texto: cadena) -> cadena {
    comprobar(texto);
    variable c: cadena = texto[aux_espacios(texto, 0)];
    si c == "\"" { retornar "cadena"; }
    si c == "{" { retornar "objeto"; }
    si c == "[" { retornar "arreglo"; }
    si c == "t" o c == "f" { retornar "booleano"; }
    si c == "n" { retornar "nulo"; }
    retornar "numero";
}
// Convierte un booleano JSON; rechaza otros valores en lugar de evaluar su veracidad.
funcion a_booleano(texto: cadena) -> booleano {
    si tipo(texto) != "booleano" { lanzar "json.a_booleano: se requiere true o false"; }
    retornar texto[aux_espacios(texto, 0)] == "t";
}
// Convierte un entero decimal a i64 acumulando en negativo para admitir el mínimo i64.
// Los límites se comprueban antes de multiplicar: el error puede capturarse en ST.
funcion a_entero(texto: cadena) -> entero {
    si tipo(texto) != "numero" { lanzar "json.a_entero: se requiere un número entero"; }
    variable i: entero = aux_espacios(texto, 0);
    variable negativo: booleano = texto[i] == "-";
    si negativo { i = i + 1; }
    variable valor: entero = 0;
    variable c: cadena = aux_letra(texto, i);
    variable digito: entero = 0;
    mientras aux_digito(c) {
        digito = buscar("0123456789", c);
        si valor < -922337203685477580 o (valor == -922337203685477580 y digito > 8) {
            lanzar "json.a_entero: fuera del rango i64";
        }
        valor = valor * 10 - digito;
        i = i + 1;
        c = aux_letra(texto, i);
    }
    si aux_espacios(texto, i) != longitud(texto) { lanzar "json.a_entero: no admite fracción ni exponente"; }
    si negativo { retornar valor; }
    si valor < -9223372036854775807 { lanzar "json.a_entero: fuera del rango i64"; }
    retornar -valor;
}
// Construye un arreglo a partir de valores ya expresados en JSON, comprobando cada elemento.
funcion arreglo(valores: lista[cadena]) -> cadena {
    variable salida: cadena = "[";
    para i en rango(0, longitud(valores)) {
        comprobar(valores[i]);
        si i > 0 { salida = salida + ","; }
        salida = salida + valores[i];
        aux_limite(salida);
    }
    salida = salida + "]";
    comprobar(salida);
    retornar salida;
}
// Construye un objeto con claves ST y valores JSON: exige igual número de claves y valores.
// Las claves repetidas son válidas; obtener() conserva la política del primer miembro.
funcion objeto(claves: lista[cadena], valores: lista[cadena]) -> cadena {
    si longitud(claves) != longitud(valores) { lanzar "json.objeto: distinta cantidad de claves y valores"; }
    variable salida: cadena = "{";
    para i en rango(0, longitud(claves)) {
        comprobar(valores[i]);
        si i > 0 { salida = salida + ","; }
        salida = salida + codificar(claves[i]) + ":" + valores[i];
        aux_limite(salida);
    }
    salida = salida + "}";
    comprobar(salida);
    retornar salida;
}

function getResults(from, to) {
	var request = new XMLHttpRequest();

	request.onreadystatechange = function() {
		if (request.readyState == 4 && request.status == 200) {
			var results = JSON.parse(request.responseText)["rust-websocket"];
			
			var cases = [];
			
			for (test_case in results) {
				cases[cases.length] = test_case;
			}
			
			cases.sort(function(a, b) {
				var array_a = a.split(".");
				var array_b = b.split(".");
				
				var x = parseInt(array_a[0]) - parseInt(array_b[0]);
				
				if (x != 0) {
					return x;
				}
				
				var y = parseInt(array_a[1]) - parseInt(array_b[1]);
				
				if (y != 0) {
					return y;
				}
				
				var z = parseInt(array_a[2]) - parseInt(array_b[2]);
				
				if (z != 0) {
					return z;
				}
				
				return 0;
			});
			
			var output = "";
			
			var passed = 0;
			var unclean = 0;
			var nonstrict = 0;
			var unimplemented = 0;
			var failed = 0;
			var informational = 0;
			
			var i;
			for (i = 0; i < cases.length; i++) {
				var test_case = cases[i];
				var classname = "";
				switch (results[test_case].behavior) {
					case "OK":
						classname += "passed";
						passed++;
						break;
					case "NON-STRICT":
						classname += "non-strict";
						nonstrict++;
						break;
					case "UNIMPLEMENTED":
						classname += "unimplemented";
						unimplemented++;
						break;
					case "FAILED":
						classname += "failed";
						failed++;
						break;
					case "INFORMATIONAL": 
						classname = "informational";
						informational++;
						break;
				}
				
				var is_unclean = results[test_case].behaviorClose != "OK";
				var file = results[test_case].reportfile;
				output += "<a href=\""+ from + "/" + file.substring(0, file.length - 4) + "html\" title=\"Case " + test_case + " (" + results[test_case].behavior + (is_unclean ? ", UNCLEAN" : "") + ")\"><span class=\"" + classname + "\">";
				
				if (is_unclean) {
					output += "<span class=\"unclean\"></span>";
					unclean++;
				}
				
				output += "</span></a>";
			}
			
			var prelude = "<p>Summary: " + i + " run, " + passed + " passed (" + unclean + " unclean), " + nonstrict + " non-strict, " + unimplemented + " unimplemented, " + failed + " failed and " + informational + " informational.</p>";
			
			document.getElementById(to).innerHTML = prelude + output;
		}
	}
	
	request.open("GET", from + "/index.json", true);
	request.send();
}

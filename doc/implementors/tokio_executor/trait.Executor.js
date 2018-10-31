(function() {var implementors = {};
implementors["tokio_current_thread"] = [{text:"impl <a class=\"trait\" href=\"tokio_executor/trait.Executor.html\" title=\"trait tokio_executor::Executor\">Executor</a> for <a class=\"struct\" href=\"tokio_current_thread/struct.CurrentThread.html\" title=\"struct tokio_current_thread::CurrentThread\">CurrentThread</a>",synthetic:false,types:["tokio_current_thread::CurrentThread"]},{text:"impl <a class=\"trait\" href=\"tokio_executor/trait.Executor.html\" title=\"trait tokio_executor::Executor\">Executor</a> for <a class=\"struct\" href=\"tokio_current_thread/struct.TaskExecutor.html\" title=\"struct tokio_current_thread::TaskExecutor\">TaskExecutor</a>",synthetic:false,types:["tokio_current_thread::TaskExecutor"]},];
implementors["tokio_executor"] = [];
implementors["tokio_threadpool"] = [{text:"impl <a class=\"trait\" href=\"tokio_executor/trait.Executor.html\" title=\"trait tokio_executor::Executor\">Executor</a> for <a class=\"struct\" href=\"tokio_threadpool/struct.Sender.html\" title=\"struct tokio_threadpool::Sender\">Sender</a>",synthetic:false,types:["tokio_threadpool::sender::Sender"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"tokio_executor/trait.Executor.html\" title=\"trait tokio_executor::Executor\">Executor</a> for &amp;'a <a class=\"struct\" href=\"tokio_threadpool/struct.Sender.html\" title=\"struct tokio_threadpool::Sender\">Sender</a>",synthetic:false,types:["tokio_threadpool::sender::Sender"]},];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
